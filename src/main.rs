use anyhow::{Context, Result};
use clap::Parser;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use load_rs::{HttpMethod, LoadTestRunner};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs;

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

    /// Custom CA certificate file (PEM format).
    #[arg(short = 'C', long = "cacert")]
    ca_cert: Option<PathBuf>,

    /// Public certificate file (PEM format).
    #[arg(short = 'E', long = "cert", requires = "key")]
    cert: Option<PathBuf>,

    /// Private key file (PEM format).
    #[arg(short = 'k', long = "key", requires = "cert")]
    key: Option<PathBuf>,
}

fn parse_http_method(s: &str) -> Result<HttpMethod, String> {
    match s.to_ascii_lowercase().as_str() {
        "get" => Ok(HttpMethod::Get),
        "post" => Ok(HttpMethod::Post),
        "put" => Ok(HttpMethod::Put),
        "delete" => Ok(HttpMethod::Delete),
        "patch" => Ok(HttpMethod::Patch),
        "head" => Ok(HttpMethod::Head),
        _ => Err(format!("'{s}' is not a valid HTTP method")),
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

async fn to_data(args: &Args) -> Result<bytes::Bytes> {
    if let Some(data) = &args.data {
        Ok(data.to_owned().into())
    } else if let Some(data_file) = &args.data_file {
        let data = fs::read(data_file).await?;
        Ok(data.into())
    } else {
        Ok(bytes::Bytes::new())
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    println!(
        "🚀🚀🚀 Sending {} requests to {} with {} concurrency 🚀🚀🚀",
        args.requests, args.url, args.concurrency
    );
    let pb = create_progress_bar(args.requests)?;
    let runner = LoadTestRunner::new(
        &args.url,
        args.requests,
        args.concurrency,
        &args.ca_cert,
        &args.cert,
        &args.key,
    )
    .await?;
    let result = runner
        .run(
            args.method,
            to_header_map(&args.header)?,
            to_data(&args).await?,
            |result| {
                pb.set_message(format!(
                    "\nSuccess: {} | Failures: {} | Avg: {:.2?}",
                    style(result.success).green(),
                    style(result.failures).red(),
                    result.avg
                ));
                pb.inc(1);
            },
        )
        .await?;
    pb.finish_with_message(format!(
        "✅ Done!\nSuccess: {} | Failures: {} | Avg: {:.2?} | P50: {:.2?} | P90: {:.2?} | P95: {:.2?}",
        style(result.success).green(), style(result.failures).red(), result.avg, result.p50, result.p90, result.p95
    ));
    Ok(())
}
