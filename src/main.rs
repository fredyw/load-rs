use anyhow::Result;
use clap::{Parser, ValueEnum};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use load_rs::LoadTestRunner;
use reqwest::header::HeaderMap;

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
    #[arg(short = 'X', long, value_enum, default_value_t = HttpMethod::Get)]
    method: HttpMethod,

    /// Custom HTTP header(s) in "key: value" format.
    /// This flag can be specified multiple times to add multiple headers.
    #[arg(short = 'H', long, action = clap::ArgAction::Append)]
    header: Vec<String>,
}

/// Defines the allowed HTTP methods that the user can specify.
#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    println!(
        "🚀🚀🚀 Sending {} requests to {} with {} concurrency 🚀🚀🚀",
        args.requests, args.url, args.concurrency
    );
    let pb = create_progress_bar(args.requests)?;
    let load_test = LoadTestRunner::new(&args.url, args.requests, args.concurrency)?;
    let result = load_test
        .run(HeaderMap::new(), |result| {
            pb.set_message(format!(
                "\nSuccess: {} | Failures: {} | Avg: {:.2?}",
                style(result.success).green(),
                style(result.failures).red(),
                result.avg
            ));
            pb.inc(1);
        })
        .await?;
    pb.finish_with_message(format!(
        "✅ Done!\nSuccess: {} | Failures: {} | Avg: {:.2?} | P50: {:.2?} | P90: {:.2?} | P95: {:.2?}",
        style(result.success).green(), style(result.failures).red(), result.avg, result.p50, result.p90, result.p95
    ));
    Ok(())
}
