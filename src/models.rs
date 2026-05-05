use anyhow::{Result, bail};
use bytes::Bytes;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

/// Defines the allowed HTTP methods that the user can specify.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_from_str() {
        assert_eq!(HttpMethod::from_str("get").unwrap(), HttpMethod::Get);
        assert_eq!(HttpMethod::from_str("POST").unwrap(), HttpMethod::Post);
        assert_eq!(HttpMethod::from_str("Put").unwrap(), HttpMethod::Put);
        assert_eq!(HttpMethod::from_str("DELETE").unwrap(), HttpMethod::Delete);
        assert_eq!(HttpMethod::from_str("patch").unwrap(), HttpMethod::Patch);
        assert_eq!(HttpMethod::from_str("HEAD").unwrap(), HttpMethod::Head);
        assert!(HttpMethod::from_str("invalid").is_err());
    }

    #[test]
    fn test_order_from_str() {
        assert_eq!(Order::from_str("sequential").unwrap(), Order::Sequential);
        assert_eq!(Order::from_str("RANDOM").unwrap(), Order::Random);
        assert!(Order::from_str("invalid").is_err());
    }

    #[test]
    fn test_stats_from_str() {
        assert_eq!(Stats::from_str("success").unwrap(), Stats::Success);
        assert_eq!(Stats::from_str("ERROR").unwrap(), Stats::Error);
        assert_eq!(Stats::from_str("All").unwrap(), Stats::All);
        assert!(Stats::from_str("invalid").is_err());
    }

    #[test]
    fn test_stats_display() {
        assert_eq!(format!("{}", Stats::Success), "Success");
        assert_eq!(format!("{}", Stats::Error), "Error");
        assert_eq!(format!("{}", Stats::All), "All");
    }

    #[test]
    fn test_unit_from_str() {
        assert_eq!(Unit::from_str("seconds").unwrap(), Unit::Seconds);
        assert_eq!(Unit::from_str("MILLISECONDS").unwrap(), Unit::Milliseconds);
        assert!(Unit::from_str("invalid").is_err());
    }

    #[test]
    fn test_load_test_result_new() {
        let res = LoadTestResult::new();
        assert_eq!(res.success, 0);
        assert_eq!(res.failures, 0);
        assert_eq!(res.completed, 0);
    }

    #[test]
    fn test_load_test_result_update_percentiles() {
        let mut res = LoadTestResult::new();
        res.durations.record(100).unwrap();
        res.durations.record(200).unwrap();
        res.durations.record(300).unwrap();
        res.total_duration = Duration::from_micros(600);

        res.update_percentiles();

        assert_eq!(res.avg, Duration::from_micros(200));
        assert!(res.p50 >= Duration::from_micros(200));
        assert!(res.p90 >= Duration::from_micros(300));
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

/// Specifies what response data to save to the output.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SaveMode {
    /// Save only response headers.
    Headers,
    /// Save only response body.
    Body,
    /// Save all response data (headers, body, version, status, duration).
    #[default]
    All,
}

impl FromStr for SaveMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "headers" => Ok(SaveMode::Headers),
            "body" => Ok(SaveMode::Body),
            "all" => Ok(SaveMode::All),
            _ => bail!("'{s}' is not a valid save mode"),
        }
    }
}

impl std::fmt::Display for SaveMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveMode::Headers => write!(f, "headers"),
            SaveMode::Body => write!(f, "body"),
            SaveMode::All => write!(f, "all"),
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

/// Configuration for a load test run.
#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    /// Target URL to send requests to.
    pub url: String,
    /// Total number of requests to send.
    pub requests: u32,
    /// Number of concurrent requests to run at a time.
    pub concurrency: u32,
    /// Specifies which requests to include in the statistics.
    pub stats: Stats,
    /// Custom CA certificate file (PEM format).
    pub ca_cert: Option<PathBuf>,
    /// Public certificate file (PEM format).
    pub cert: Option<PathBuf>,
    /// Private key file (PEM format).
    pub key: Option<PathBuf>,
    /// Allows insecure connections by skipping TLS certificate verification.
    pub insecure: bool,
    /// Request timeout in seconds.
    pub timeout: Option<u64>,
    /// Custom user agent.
    pub user_agent: Option<String>,
    /// Proxy server URL.
    pub proxy: Option<String>,
    /// Quiet mode: suppress progress updates.
    pub quiet: bool,
    /// Specifies what to save in the response output.
    pub save_mode: SaveMode,
    /// Disables HTTP keep-alive.
    pub disable_keepalive: bool,
}

/// Events emitted during the load test.
pub enum LoadTestEvent<'a> {
    /// Emitted when an individual request finishes.
    RequestFinished(RequestResult),
    /// Emitted periodically with aggregated statistics.
    ProgressUpdate(&'a LoadTestResult),
}
