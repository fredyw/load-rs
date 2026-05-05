use anyhow::{Result, bail};
use bytes::Bytes;
use reqwest::header::HeaderMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

/// Defines the allowed HTTP methods that the user can specify.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
}

impl FromStr for HttpMethod {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
}

/// Represents the aggregated results of a load test run.
#[derive(Debug, Clone)]
pub struct LoadTestResult {
    ///  Total number of successful requests.
    pub success: u32,
    /// Total number of failed requests.
    pub failures: u32,
    /// Total number of completed requests (success + failures).
    pub completed: u32,
    /// Cumulative duration of all successful requests combined.
    pub total_duration: Duration,
    /// A histogram of individual response durations (in microseconds) for each successful request.
    pub durations: hdrhistogram::Histogram<u64>,
    /// The average response time for successful requests.
    pub avg: Duration,
    /// The minimum response time for successful requests.
    pub min: Duration,
    /// The maximum response time for successful requests.
    pub max: Duration,
    /// The 50th percentile (median) response time for successful requests.
    pub p50: Duration,
    /// The 90th percentile response time for successful requests.
    pub p90: Duration,
    /// The 95th percentile response time for successful requests.
    pub p95: Duration,
    /// The 99th percentile response time for successful requests.
    pub p99: Duration,
    /// Total time elapsed for the load test.
    pub elapsed: Duration,
    /// Total requests per second.
    pub rps: f64,
    /// Successful requests percentage.
    pub success_rate: f64,
    /// Failed requests percentage.
    pub failure_rate: f64,
}

impl LoadTestResult {
    pub(crate) fn new() -> Self {
        LoadTestResult {
            success: 0,
            failures: 0,
            completed: 0,
            total_duration: Duration::default(),
            durations: hdrhistogram::Histogram::<u64>::new(3).unwrap(),
            avg: Duration::default(),
            min: Duration::default(),
            max: Duration::default(),
            p50: Duration::default(),
            p90: Duration::default(),
            p95: Duration::default(),
            p99: Duration::default(),
            elapsed: Duration::default(),
            rps: 0.0,
            success_rate: 0.0,
            failure_rate: 0.0,
        }
    }

    pub(crate) fn update_percentiles(&mut self) {
        if !self.durations.is_empty() {
            self.p50 = Duration::from_micros(self.durations.value_at_percentile(50.0));
            self.p90 = Duration::from_micros(self.durations.value_at_percentile(90.0));
            self.p95 = Duration::from_micros(self.durations.value_at_percentile(95.0));
            self.p99 = Duration::from_micros(self.durations.value_at_percentile(99.0));
            self.avg = self.total_duration / self.durations.len() as u32;
        }
    }
}

/// Specifies the order in which to process request body files from a directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    /// Process files in alphabetical order (default).
    Sequential,
    /// Process files in a random order.
    Random,
}

impl FromStr for Order {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "sequential" => Ok(Order::Sequential),
            "random" => Ok(Order::Random),
            _ => bail!("'{s}' is not a valid read order"),
        }
    }
}

/// Specifies which requests to include in the statistics.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Stats {
    /// Only include successful requests in the statistics.
    Success,
    /// Only include failed requests in the statistics.
    Error,
    /// Include all requests (successful and failed) in the statistics.
    All,
}

impl FromStr for Stats {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "success" => Ok(Stats::Success),
            "error" => Ok(Stats::Error),
            "all" => Ok(Stats::All),
            _ => bail!("'{s}' is not a valid stats"),
        }
    }
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stats::Success => write!(f, "Success"),
            Stats::Error => write!(f, "Error"),
            Stats::All => write!(f, "All"),
        }
    }
}

/// Unit of measurement (seconds or milliseconds).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum Unit {
    Seconds,
    #[default]
    Milliseconds,
}

impl FromStr for Unit {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "seconds" => Ok(Unit::Seconds),
            "milliseconds" => Ok(Unit::Milliseconds),
            _ => bail!("'{s}' is not a valid unit"),
        }
    }
}

/// Represents the data received from a successful HTTP request.
#[derive(Debug, Clone)]
pub struct ResponseData {
    /// HTTP version used.
    pub version: reqwest::Version,
    /// HTTP status code.
    pub status: reqwest::StatusCode,
    /// HTTP headers.
    pub headers: HeaderMap,
    /// Response body bytes.
    pub body: Bytes,
}

/// Represents the source for the HTTP request body or bodies.
///
/// This enum allows for specifying the body data directly as a string, from a single file, or from
/// a directory containing multiple files to be used in multiple requests.
pub enum Body {
    /// The request body is provided directly as an in-memory byte slice.
    Data(Bytes),
    /// The request body will be read from a single specified file.
    DataFile(PathBuf),
}

/// The raw result of an individual request.
#[derive(Debug, Clone)]
pub struct RequestResult {
    /// The iteration number of the request.
    pub iteration: u64,
    /// The time taken to complete the request.
    pub duration: Duration,
    /// Whether the request was successful.
    pub success: bool,
    /// The HTTP status code returned, if any.
    pub status_code: Option<reqwest::StatusCode>,
    /// The error message, if the request failed.
    pub error: Option<String>,
}

/// Events emitted during the load test.
pub enum LoadTestEvent<'a> {
    /// Emitted when an individual request finishes.
    RequestFinished(RequestResult),
    /// Emitted periodically with aggregated statistics.
    ProgressUpdate(&'a LoadTestResult),
}
