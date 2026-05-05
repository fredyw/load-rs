use anyhow::{Context, Result};
use bytes::Bytes;
use clap::Parser;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use load_rs::{Body, HttpMethod, LoadTestRunner, Order, Stats, Unit};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::path::PathBuf;
use std::str::FromStr;

/// load-rs: A simple load testing tool written in Rust.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target URL to send requests to.
    url: String,

    /// Total number of requests to send.
    #[arg(short = 'n', long)]
    requests: u32,

    /// Number of concurrent requests to run at a time.
    #[arg(short = 'c', long)]
    concurrency: u32,

    /// HTTP method to use for the requests.
    #[arg(short = 'X', long, default_value = "get")]
    method: HttpMethod,

    /// Custom HTTP header(s) in "key: value" format. Can be repeated.
    #[arg(short = 'H', long, action = clap::ArgAction::Append)]
    header: Vec<String>,

    /// Request body as a string.
    #[arg(short = 'd', long, group = "request_body")]
    data: Option<String>,

    /// File to read the request body from.
    #[arg(short = 'D', long = "data-file", group = "request_body")]
    data_file: Option<PathBuf>,

    /// Directory of files to use as request bodies.
    #[arg(short = 'i', long = "data-dir", group = "request_body")]
    data_dir: Option<PathBuf>,

    /// Request manifest file (JSON Lines format).
    #[arg(short = 'm', long = "manifest-file", group = "request_body")]
    manifest_file: Option<PathBuf>,

    /// Custom CA certificate file (PEM format).
    #[arg(short = 'C', long = "cacert")]
    ca_cert: Option<PathBuf>,

    /// Public certificate file (PEM format).
    #[arg(short = 'E', long, requires = "key")]
    cert: Option<PathBuf>,

    /// Private key file (PEM format).
    #[arg(short = 'k', long, requires = "cert")]
    key: Option<PathBuf>,

    /// Allows insecure connections by skipping TLS certificate verification.
    #[arg(short = 'I', long)]
    insecure: bool,

    /// Order to process files from --data-dir or --manifest-file.
    #[arg(short = 'O', long, default_value = "sequential", requires = "data_dir")]
    order: Order,

    /// Directory to save responses to.
    #[arg(short = 'o', long = "output-dir")]
    output_dir: Option<PathBuf>,

    /// Performs a single request and dumps the response.
    #[arg(short = 'G', long)]
    debug: bool,

    /// Specifies which requests to include in the statistics.
    #[arg(short = 's', long, default_value = "success")]
    stats: Stats,

    /// Request timeout in seconds.
    #[arg(short = 't', long)]
    timeout: Option<u64>,

    /// Custom user agent.
    #[arg(short = 'A', long = "user-agent")]
    user_agent: Option<String>,

    /// Unit of measurement (seconds or milliseconds).
    #[arg(short = 'u', long, default_value = "milliseconds")]
    unit: Unit,
}

fn to_header_map(headers: &[String]) -> Result<HeaderMap> {
    headers
        .iter()
        .map(|header| {
            let (key, value) = header
                .split_once(':')
                .context(format!("Invalid header format: {header}"))?;
            let name = HeaderName::from_str(key.trim())?;
            let value = HeaderValue::from_str(value.trim())?;
            Ok((name, value))
        })
        .collect()
}

fn to_body(args: &Args) -> Body {
    if let Some(data) = &args.data {
        Body::Data(Bytes::copy_from_slice(data.as_bytes()))
    } else if let Some(data_file) = &args.data_file {
        Body::DataFile(data_file.to_owned())
    } else {
        Body::Data(Bytes::new())
    }
}

fn format_duration(duration: std::time::Duration, unit: Unit) -> String {
    match unit {
        Unit::Seconds => format!("{:.2}s", duration.as_secs_f64()),
        Unit::Milliseconds => format!("{:.2}ms", duration.as_secs_f64() * 1000.0),
    }
}

fn create_progress_bar(len: u32) -> Result<ProgressBar> {
    let pb = ProgressBar::new(len as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise:.yellow}] [{bar:40.cyan/blue}] {pos:.blue}/{len:.blue} ({percent:.blue}%)\n\n{msg}")?
            .progress_chars("#>-"),
    );
    pb.set_position(0);
    Ok(pb)
}

fn format_progress_message(args: &Args, result: &load_rs::LoadTestResult) -> String {
    format!(
        concat!(
            "{}\n",
            "  URL: {}\n",
            "  Concurrency: {}\n",
            "  RPS: {}\n",
            "  Duration: {}\n\n",
            "{}\n",
            "  Total: {}\n",
            "  Success: {} ({:.1}%)\n",
            "  Failures: {} ({:.1}%)\n\n",
            "{} (Filter: {}):\n",
            "  Avg: {}\n",
            "  Min: {}\n",
            "  Max: {}\n",
            "  P50: {}\n",
            "  P90: {}\n",
            "  P95: {}\n",
            "  P99: {}",
        ),
        style("Overview:").bold(),
        style(&args.url).cyan().underlined(),
        style(args.concurrency).yellow(),
        style(format!("{:.1}", result.rps)).cyan(),
        style(format_duration(result.elapsed, args.unit)).yellow(),
        style("Requests:").bold(),
        style(result.completed).blue(),
        style(result.success).green(),
        result.success_rate,
        style(result.failures).red(),
        result.failure_rate,
        style("Latency").bold(),
        match args.stats {
            load_rs::Stats::Success => style(args.stats).green(),
            load_rs::Stats::Error => style(args.stats).red(),
            load_rs::Stats::All => style(args.stats).blue(),
        },
        style(format_duration(result.avg, args.unit)).yellow(),
        style(format_duration(result.min, args.unit)).yellow(),
        style(format_duration(result.max, args.unit)).yellow(),
        style(format_duration(result.p50, args.unit)).yellow(),
        style(format_duration(result.p90, args.unit)).yellow(),
        style(format_duration(result.p95, args.unit)).yellow(),
        style(format_duration(result.p99, args.unit)).yellow(),
    )
}

async fn run(runner: &LoadTestRunner, args: &Args) -> Result<()> {
    let pb = create_progress_bar(args.requests)?;
    let result = if let Some(data_dir) = &args.data_dir {
        runner
            .run_from_dir(
                args.method,
                Some(to_header_map(&args.header)?),
                data_dir,
                args.order,
                args.output_dir.as_deref(),
                |result| {
                    pb.set_message(format_progress_message(args, result));
                    pb.set_position(result.completed as u64);
                },
            )
            .await?
    } else if let Some(manifest_file) = &args.manifest_file {
        runner
            .run_from_manifest(
                args.method,
                manifest_file,
                args.order,
                args.output_dir.as_deref(),
                |result| {
                    pb.set_message(format_progress_message(args, result));
                    pb.set_position(result.completed as u64);
                },
            )
            .await?
    } else {
        runner
            .run(
                args.method,
                Some(to_header_map(&args.header)?),
                Some(to_body(args)),
                args.output_dir.as_deref(),
                |result| {
                    pb.set_message(format_progress_message(args, result));
                    pb.set_position(result.completed as u64);
                },
            )
            .await?
    };
    pb.finish_and_clear();
    println!(
        "{} {} in {}",
        style("✓").green().bold(),
        style("Test Completed").bold(),
        style(format_duration(result.elapsed, args.unit)).yellow()
    );
    println!();

    println!("{}", style("Overview:").bold());
    println!("  URL: {}", style(&args.url).cyan().underlined());
    println!("  Concurrency: {}", style(args.concurrency).yellow());
    println!("  RPS: {}", style(format!("{:.1}", result.rps)).cyan());
    println!(
        "  Duration: {}",
        style(format_duration(result.elapsed, args.unit)).yellow()
    );

    println!("\n{}", style("Requests:").bold());
    println!("  Total: {}", style(result.completed).blue());
    println!(
        "  Success: {} ({:.1}%)",
        style(result.success).green(),
        result.success_rate
    );
    println!(
        "  Failures: {} ({:.1}%)",
        style(result.failures).red(),
        result.failure_rate
    );

    println!(
        "\n{} (Filter: {}):",
        style("Latency").bold(),
        match args.stats {
            load_rs::Stats::Success => style(args.stats).green(),
            load_rs::Stats::Error => style(args.stats).red(),
            load_rs::Stats::All => style(args.stats).blue(),
        }
    );
    println!(
        "  Avg: {}",
        style(format_duration(result.avg, args.unit)).yellow()
    );
    println!(
        "  Min: {}",
        style(format_duration(result.min, args.unit)).yellow()
    );
    println!(
        "  Max: {}",
        style(format_duration(result.max, args.unit)).yellow()
    );
    println!(
        "  P50: {}",
        style(format_duration(result.p50, args.unit)).yellow()
    );
    println!(
        "  P90: {}",
        style(format_duration(result.p90, args.unit)).yellow()
    );
    println!(
        "  P95: {}",
        style(format_duration(result.p95, args.unit)).yellow()
    );
    println!(
        "  P99: {}",
        style(format_duration(result.p99, args.unit)).yellow()
    );
    Ok(())
}

async fn debug(runner: &LoadTestRunner, args: &Args) -> Result<()> {
    let response = if let Some(data_dir) = &args.data_dir {
        runner
            .debug_from_dir(
                args.method,
                Some(to_header_map(&args.header)?),
                data_dir,
                args.order,
            )
            .await?
    } else if let Some(manifest_file) = &args.manifest_file {
        runner
            .debug_from_manifest(args.method, manifest_file, args.order)
            .await?
    } else {
        runner
            .debug(
                args.method,
                Some(to_header_map(&args.header)?),
                Some(to_body(args)),
            )
            .await?
    };
    println!("{:?} {}", response.version(), response.status());
    for (name, value) in response.headers() {
        println!("{}: {}", name, value.to_str().unwrap_or(""));
    }
    println!();
    let body = response.text().await?;
    println!("{body}");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let runner = LoadTestRunner::new(
        &args.url,
        args.requests,
        args.concurrency,
        args.stats,
        args.ca_cert.as_deref(),
        args.cert.as_deref(),
        args.key.as_deref(),
        args.insecure,
        args.timeout,
        args.user_agent.as_deref(),
    )
    .await?;
    if args.debug {
        debug(&runner, &args).await?;
    } else {
        run(&runner, &args).await?;
    }
    Ok(())
}
