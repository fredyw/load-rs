use anyhow::Result;
use clap::{Parser, ValueEnum};
use console::style;
use futures::{StreamExt, stream};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::time::{Duration, Instant};

/// load-rs: A simple load testing tool written in Rust.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target URL to send requests to.
    url: String,

    /// Total number of requests to send.
    #[arg(short = 'n', long)]
    requests: u64,

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

fn create_progress_bar(len: u64) -> Result<ProgressBar> {
    let pb = ProgressBar::new(len);
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
    let mut successful_requests = 0;
    let mut failed_requests = 0;
    let mut total_duration = Duration::new(0, 0);
    let mut durations: Vec<Duration> = Vec::with_capacity(args.requests as usize);
    let mut completed_requests = 0;

    println!(
        "🚀🚀🚀 Sending {} requests to {} with {} concurrency 🚀🚀🚀",
        args.requests, args.url, args.concurrency
    );

    let pb = create_progress_bar(args.requests)?;
    let client = Client::builder().use_rustls_tls().build()?;

    let mut stream = stream::iter(0..args.requests)
        .map(|_| {
            let client = client.clone();
            let url = args.url.clone();
            async move {
                let request_start_time = Instant::now();
                let response = client.get(url).send().await;
                let request_duration = request_start_time.elapsed();
                let is_success = match response {
                    Ok(res) if res.status().is_success() => true,
                    _ => false,
                };
                (is_success, request_duration)
            }
        })
        .buffer_unordered(args.concurrency as usize);

    while let Some((is_success, request_duration)) = stream.next().await {
        if is_success {
            successful_requests += 1;
        } else {
            failed_requests += 1;
        }
        completed_requests += 1;

        total_duration += request_duration;
        durations.push(request_duration);
        let avg_duration = total_duration / completed_requests;

        pb.set_message(format!(
            "\nSuccess: {} | Failures: {} | Avg: {:.2?}",
            style(successful_requests).green(),
            style(failed_requests).red(),
            avg_duration
        ));
        pb.inc(1);
    }

    let (p50, p90, p95) = if !durations.is_empty() {
        durations.sort();
        let p50_index = (durations.len() as f64 * 0.50) as usize;
        let p90_index = (durations.len() as f64 * 0.90) as usize;
        let p95_index = (durations.len() as f64 * 0.95) as usize;
        let p50_val = durations.get(p50_index).cloned().unwrap_or_default();
        let p90_val = durations.get(p90_index).cloned().unwrap_or_default();
        let p95_val = durations.get(p95_index).cloned().unwrap_or_default();
        (p50_val, p90_val, p95_val)
    } else {
        (
            Duration::default(),
            Duration::default(),
            Duration::default(),
        )
    };

    let final_avg_duration = if args.requests > 0 {
        total_duration / args.requests as u32
    } else {
        Duration::new(0, 0)
    };
    pb.finish_with_message(format!(
        "✅ Done!\nSuccess: {} | Failures: {} | Avg: {:.2?} | P50: {:.2?} | P90: {:.2?} | P95: {:.2?}",
        style(successful_requests).green(), style(failed_requests).red(), final_avg_duration, p50, p90, p95
    ));
    Ok(())
}
