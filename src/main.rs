use anyhow::{Context, Result, bail};
use bytes::Bytes;
use clap::Parser;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use load_rs::{Body, HttpMethod, LoadTestRunner, Order};
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
    #[arg(short = 'X', long, value_parser = parse_http_method, default_value = "get")]
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
    #[arg(short = 'E', long = "cert", requires = "key")]
    cert: Option<PathBuf>,

    /// Private key file (PEM format).
    #[arg(short = 'k', long = "key", requires = "cert")]
    key: Option<PathBuf>,

    /// Allows insecure connections by skipping TLS certificate verification.
    #[arg(short = 'I', long)]
    insecure: Option<bool>,

    /// Order to process files from --data-dir or --manifest-file.
    #[arg(short = 'O', long, value_parser = parse_order, default_value = "sequential", requires = "data_dir")]
    order: Order,

    /// Directory to save responses to.
    #[arg(short = 'o', long = "output-dir")]
    output_dir: Option<PathBuf>,

    /// Performs a single request and dumps the response.
    #[arg(short = 'G', long)]
    debug: bool,
}

fn parse_http_method(s: &str) -> Result<HttpMethod> {
    match s.to_ascii_lowercase().as_str() {
        "get" => Ok(HttpMethod::Get),
        "post" => Ok(HttpMethod::Post),
        "put" => Ok(HttpMethod::Put),
        "delete" => Ok(HttpMethod::Delete),
        "patch" => Ok(HttpMethod::Patch),
        "head" => Ok(HttpMethod::Head),
        _ => bail!("'{s}' is not a valid HTTP method"),
    }
}

fn parse_order(s: &str) -> Result<Order> {
    match s.to_ascii_lowercase().as_str() {
        "sequential" => Ok(Order::Sequential),
        "random" => Ok(Order::Random),
        _ => bail!("'{s}' is not a valid read order"),
    }
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
        Body::Data(data.to_owned().into())
    } else if let Some(data_file) = &args.data_file {
        Body::DataFile(data_file.to_owned())
    } else {
        Body::Data(Bytes::new())
    }
}

fn create_progress_bar(len: u32) -> Result<ProgressBar> {
    let pb = ProgressBar::new(len as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")?
            .progress_chars("#>-"),
    );
    pb.set_position(0);
    Ok(pb)
}

async fn run(runner: &LoadTestRunner, args: &Args) -> Result<()> {
    println!(
        "🚀🚀🚀 Sending {} requests to {} with {} concurrency 🚀🚀🚀",
        args.requests, args.url, args.concurrency
    );
    let pb = create_progress_bar(args.requests)?;
    let result = if let Some(data_dir) = &args.data_dir {
        runner
            .run_from_dir(
                args.method,
                Some(to_header_map(&args.header)?),
                data_dir,
                args.order,
                &args.output_dir,
                |result| {
                    pb.set_message(format!(
                        "\nSuccess: {} | Failures: {} | RPS: {:.2?} | Avg: {:.2?}",
                        style(result.success).green(),
                        style(result.failures).red(),
                        result.rps,
                        result.avg
                    ));
                    pb.inc(1);
                },
            )
            .await?
    } else if let Some(manifest_file) = &args.manifest_file {
        runner
            .run_from_manifest(
                args.method,
                manifest_file,
                args.order,
                &args.output_dir,
                |result| {
                    pb.set_message(format!(
                        "\nSuccess: {} | Failures: {} | RPS: {:.2?} | Avg: {:.2?}",
                        style(result.success).green(),
                        style(result.failures).red(),
                        result.rps,
                        result.avg
                    ));
                    pb.inc(1);
                },
            )
            .await?
    } else {
        runner
            .run(
                args.method,
                Some(to_header_map(&args.header)?),
                Some(to_body(args)),
                &args.output_dir,
                |result| {
                    pb.set_message(format!(
                        "\nSuccess: {} | Failures: {} | RPS: {:.2?} | Avg: {:.2?} | Min: {:.2?} | Max: {:.2?}",
                        style(result.success).green(),
                        style(result.failures).red(),
                        result.rps,
                        result.avg,
                        result.min,
                        result.max
                    ));
                    pb.inc(1);
                },
            )
            .await?
    };
    pb.finish_with_message(format!(
        "✅ Done!\nSuccess: {} | Failures: {} | RPS: {:.2?} | Avg: {:.2?} | Min: {:.2?} | Max: {:.2?} | P50: {:.2?} | P90: {:.2?} | P95: {:.2?}",
        style(result.success).green(), style(result.failures).red(), result.rps, result.avg, result.min, result.max, result.p50, result.p90, result.p95
    ));
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
        &args.ca_cert,
        &args.cert,
        &args.key,
        &args.insecure,
    )
    .await?;
    if args.debug {
        debug(&runner, &args).await?;
    } else {
        run(&runner, &args).await?;
    }
    Ok(())
}
