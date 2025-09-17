use anyhow::Result;
use clap::{Parser, ValueEnum};

/// load-rs: A simple load testing tool written in Rust.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target URL to send requests to.
    url: String,

    /// Total number of requests to send.
    #[arg(short = 'n', long)]
    requests: usize,

    /// Number of concurrent requests to run at a time.
    #[arg(short = 'c', long)]
    concurrency: usize,

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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    Ok(())
}
