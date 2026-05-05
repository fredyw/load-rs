use anyhow::{Result, bail};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use bytes::Bytes;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Certificate, Client, Identity, Response};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::str::{self, FromStr};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::task::JoinSet;

/// A load test runner responsible for configuring and executing a load test.
#[derive(Debug, Clone)]
pub struct LoadTestRunner {
    /// Target URL to send requests to.
    pub url: String,

    /// Total number of requests to send.
    pub requests: u32,

    /// Number of concurrent requests to run at a time.
    pub concurrency: u32,

    /// Specifies which requests to include in the statistics.
    pub stats: Stats,

    /// HTTP client.
    client: Client,
}

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

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stats::Success => write!(f, "Success"),
            Stats::Error => write!(f, "Error"),
            Stats::All => write!(f, "All"),
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

impl LoadTestResult {
    fn new() -> Self {
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

    fn update_percentiles(&mut self) {
        if !self.durations.is_empty() {
            self.p50 = Duration::from_micros(self.durations.value_at_percentile(50.0));
            self.p90 = Duration::from_micros(self.durations.value_at_percentile(90.0));
            self.p95 = Duration::from_micros(self.durations.value_at_percentile(95.0));
            self.p99 = Duration::from_micros(self.durations.value_at_percentile(99.0));
            self.avg = self.total_duration / self.durations.len() as u32;
        }
    }
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

/// Specifies the order in which to process request body files from a directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    /// Process files in alphabetical order (default).
    Sequential,

    /// Process files in a random order.
    Random,
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

/// JSON representation of manifest request file.
#[derive(Debug, Clone, Deserialize)]
struct RequestTemplate {
    #[serde(default)]
    headers: HashMap<String, String>,
    body: Option<String>,
    binary_body: Option<String>,
}

enum WorkerResult {
    StatsOnly {
        success: bool,
        duration: Duration,
    },
    WithResponse {
        res: Result<ResponseData>,
        duration: Duration,
        iteration: u64,
        base_file_name: Option<OsString>,
    },
}

impl LoadTestRunner {
    /// Creates a new `LoadTestRunner` with the specified configuration.
    ///
    /// # Parameters
    ///
    /// * `url`: Target URL to send requests to.
    /// * `requests`: Total number of requests to send.
    /// * `concurrency`: Number of concurrent requests to run at a time.
    /// * `stats`: Specifies which requests to include in the statistics.
    /// * `ca_cert`: Custom CA certificate file (PEM format).
    /// * `cert`: Public certificate file (PEM format).
    /// * `key`: Private key file (PEM format).
    ///
    /// # Returns
    /// A `Result` containing the new `LoadTestRunner` instance if successful.
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        url: &str,
        requests: u32,
        concurrency: u32,
        stats: Stats,
        ca_cert: Option<&Path>,
        cert: Option<&Path>,
        key: Option<&Path>,
        insecure: bool,
        timeout: Option<u64>,
        user_agent: Option<&str>,
    ) -> Result<Self> {
        if url.is_empty() {
            bail!("URL cannot be empty");
        }
        if requests == 0 {
            bail!("Number of requests cannot be zero");
        }
        if concurrency == 0 {
            bail!("Number of concurrency cannot be zero");
        }
        if concurrency > requests {
            bail!(
                "Number of concurrency: {concurrency} must be less than or equal to number of requests: {requests}"
            );
        }
        let mut builder = Client::builder()
            .use_rustls_tls()
            .danger_accept_invalid_certs(insecure)
            .tcp_nodelay(true)
            .pool_max_idle_per_host(concurrency as usize);
        if let Some(t) = timeout {
            builder = builder.timeout(Duration::from_secs(t));
        }
        if let Some(ua) = user_agent {
            builder = builder.user_agent(ua);
        }
        if let Some(ca_cert_path) = ca_cert {
            if !ca_cert_path.is_file() {
                bail!(
                    "CA certificate '{}' does not exist or is not a file",
                    ca_cert_path.display()
                );
            }
            let bytes = fs::read(ca_cert_path).await?;
            let ca_cert_bytes = Certificate::from_pem(&bytes)?;
            builder = builder.add_root_certificate(ca_cert_bytes);
        }
        if let (Some(cert_path), Some(key_path)) = (cert, key) {
            builder = builder.identity(Self::create_identity(cert_path, key_path).await?);
        }
        Ok(LoadTestRunner {
            url: url.to_owned(),
            requests,
            concurrency,
            stats,
            client: builder.build()?,
        })
    }

    /// Executes the load test and streams progress updates via a callback.
    ///
    /// This is the main method for running the test. It sends the configured number of requests
    /// concurrently to the target URL. After each request completes, it invokes the `in_progress`
    /// callback with the current, cumulative statistics.
    ///
    /// # Parameters
    ///
    /// * `method`: HTTP method (GET, POST, etc.) to use.
    /// * `header`: A `reqwest::header::HeaderMap` containing custom HTTP headers to be sent with
    ///   each request.
    /// * `body`: Request body. It can be in-memory byte slice or a file that contains a request
    ///   body.
    /// * `in_progress`: A callback function that is invoked after each request completes.
    ///   It receives a reference to the `LoadTestResult` struct, allowing for real-time progress
    ///   reporting.
    ///
    /// # Returns
    ///
    /// Upon completion of all requests, it returns a `Result` containing the final `LoadTestResult`
    /// with the complete summary of the test run.
    pub async fn run<T>(
        &self,
        method: HttpMethod,
        header: Option<HeaderMap>,
        body: Option<Body>,
        output_dir: Option<&Path>,
        in_progress: T,
    ) -> Result<LoadTestResult>
    where
        T: Fn(&LoadTestResult),
    {
        let body = Arc::new(Self::get_data(body.unwrap_or(Body::Data(Bytes::new()))).await?);
        let headers = Arc::new(header.unwrap_or_default());
        let save_response = output_dir.is_some();
        let (tx, rx) = mpsc::channel(self.concurrency as usize);
        let runner = Arc::new(self.clone());
        let counter = Arc::new(AtomicU64::new(0));
        let base_req = self.build_request(method, (*headers).clone(), (*body).clone())?;
        for _ in 0..self.concurrency {
            let tx = tx.clone();
            let runner = Arc::clone(&runner);
            let counter = Arc::clone(&counter);
            let base_req = base_req.try_clone().unwrap();
            tokio::spawn(async move {
                loop {
                    let i = counter.fetch_add(1, Ordering::Relaxed);
                    if i >= runner.requests as u64 {
                        break;
                    }
                    let req = base_req.try_clone().unwrap();
                    let result = runner.timed_request(req, i, None, save_response).await;
                    if tx.send(result).await.is_err() {
                        break;
                    }
                }
            });
        }
        drop(tx);
        self.process_results(rx, in_progress, output_dir).await
    }

    /// Executes the load test with request bodies from files in a directory and streams progress
    /// updates via a callback.
    ///
    /// This is the main method for running the test. It sends the configured number of requests
    /// concurrently to the target URL. After each request completes, it invokes the `in_progress`
    /// callback with the current, cumulative statistics.
    ///
    /// # Parameters
    ///
    /// * `method`: HTTP method (GET, POST, etc.) to use.
    /// * `header`: A `reqwest::header::HeaderMap` containing custom HTTP headers to be sent with
    ///   each request.
    /// * `data_dir`: Directory of files to use as request bodies.
    /// * `order`: Order to process files from the `data_dir`.
    /// * `output_dir`: Directory to save responses to.
    /// * `in_progress`: A callback function that is invoked after each request completes.
    ///   It receives a reference to the `LoadTestResult` struct, allowing for real-time progress
    ///   reporting.
    ///
    /// # Returns
    ///
    /// Upon completion of all requests, it returns a `Result` containing the final `LoadTestResult`
    /// with the complete summary of the test run.
    pub async fn run_from_dir<T>(
        &self,
        method: HttpMethod,
        header: Option<HeaderMap>,
        data_dir: &Path,
        order: Order,
        output_dir: Option<&Path>,
        in_progress: T,
    ) -> Result<LoadTestResult>
    where
        T: Fn(&LoadTestResult),
    {
        if method == HttpMethod::Get || method == HttpMethod::Head {
            bail!("HTTP method '{:?}' not supported", method);
        }
        let mut file_names = Self::get_file_names(data_dir).await?;
        // Sort the file names to make it deterministic.
        file_names.sort();
        let mut bodies = Vec::new();
        for path in &file_names {
            let body = fs::read(path).await?;
            bodies.push((
                Arc::new(Bytes::from(body)),
                path.file_stem().map(|f| f.to_owned()),
            ));
        }
        let save_response = output_dir.is_some();
        let mut reqs = Vec::new();
        for (body, base_file_name) in bodies.iter() {
            let req =
                self.build_request(method, header.clone().unwrap_or_default(), (**body).clone())?;
            reqs.push((req, base_file_name.clone()));
        }
        let reqs = Arc::new(reqs);
        let (tx, rx) = mpsc::channel(self.concurrency as usize);
        let runner = Arc::new(self.clone());
        let counter = Arc::new(AtomicU64::new(0));
        for _ in 0..self.concurrency {
            let tx = tx.clone();
            let reqs = Arc::clone(&reqs);
            let runner = Arc::clone(&runner);
            let counter = Arc::clone(&counter);
            tokio::spawn(async move {
                loop {
                    let i = counter.fetch_add(1, Ordering::Relaxed);
                    if i >= runner.requests as u64 {
                        break;
                    }
                    let index = match order {
                        Order::Sequential => i as usize % reqs.len(),
                        Order::Random => rand::random_range(0..reqs.len()),
                    };
                    let (req, base_file_name) = &reqs[index];
                    let req = req.try_clone().unwrap();
                    let base_file_name = base_file_name.clone();
                    let result = runner
                        .timed_request(req, i, base_file_name, save_response)
                        .await;
                    if tx.send(result).await.is_err() {
                        break;
                    }
                }
            });
        }
        drop(tx);
        self.process_results(rx, in_progress, output_dir).await
    }

    /// Executes the load test with a request manifest file and streams progress updates via a
    /// callback.
    ///
    /// This is the main method for running the test. It sends the configured number of requests
    /// concurrently to the target URL. After each request completes, it invokes the `in_progress`
    /// callback with the current, cumulative statistics.
    ///
    /// # Parameters
    ///
    /// * `method`: HTTP method (GET, POST, etc.) to use.
    /// * `manifest_file`: A manifest file.
    /// * `order`: Order to process request from the `manifest_file`.
    /// * `output_dir`: Directory to save responses to.
    /// * `in_progress`: A callback function that is invoked after each request completes.
    ///   It receives a reference to the `LoadTestResult` struct, allowing for real-time progress
    ///   reporting.
    ///
    /// # Returns
    ///
    /// Upon completion of all requests, it returns a `Result` containing the final `LoadTestResult`
    /// with the complete summary of the test run.
    pub async fn run_from_manifest<T>(
        &self,
        method: HttpMethod,
        manifest_file: &Path,
        order: Order,
        output_dir: Option<&Path>,
        in_progress: T,
    ) -> Result<LoadTestResult>
    where
        T: Fn(&LoadTestResult),
    {
        let file = File::open(manifest_file).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut templates = Vec::new();
        while let Some(line) = lines.next_line().await? {
            let template: RequestTemplate = serde_json::from_str(&line)?;
            let mut headers = HeaderMap::new();
            for (name, value) in &template.headers {
                headers.insert(HeaderName::from_str(name)?, HeaderValue::from_str(value)?);
            }
            let body = if let Some(body) = &template.body {
                Bytes::from(body.clone())
            } else if let Some(base64_body) = &template.binary_body {
                Bytes::from(BASE64_STANDARD.decode(base64_body)?)
            } else {
                Bytes::new()
            };
            templates.push((Arc::new(headers), Arc::new(body)));
        }
        let save_response = output_dir.is_some();
        let mut reqs = Vec::new();
        for (headers, body) in templates.iter() {
            let req = self.build_request(method, (**headers).clone(), (**body).clone())?;
            reqs.push(req);
        }
        let reqs = Arc::new(reqs);
        let (tx, rx) = mpsc::channel(self.concurrency as usize);
        let runner = Arc::new(self.clone());
        let counter = Arc::new(AtomicU64::new(0));
        for _ in 0..self.concurrency {
            let tx = tx.clone();
            let reqs = Arc::clone(&reqs);
            let runner = Arc::clone(&runner);
            let counter = Arc::clone(&counter);
            tokio::spawn(async move {
                loop {
                    let i = counter.fetch_add(1, Ordering::Relaxed);
                    if i >= runner.requests as u64 {
                        break;
                    }
                    let index = match order {
                        Order::Sequential => i as usize % reqs.len(),
                        Order::Random => rand::random_range(0..reqs.len()),
                    };
                    let req = reqs[index].try_clone().unwrap();
                    let result = runner.timed_request(req, i, None, save_response).await;
                    if tx.send(result).await.is_err() {
                        break;
                    }
                }
            });
        }
        drop(tx);
        self.process_results(rx, in_progress, output_dir).await
    }

    /// Executes a single request for debugging.
    ///
    /// # Parameters
    ///
    /// * `method`: HTTP method (GET, POST, etc.) to use.
    /// * `header`: A `reqwest::header::HeaderMap` containing custom HTTP headers to be sent with
    ///   each request.
    /// * `body`: Request body. It can be in-memory byte slice or a file that contains a request
    ///   body.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing `reqwest::Response`.
    pub async fn debug(
        &self,
        method: HttpMethod,
        header: Option<HeaderMap>,
        body: Option<Body>,
    ) -> Result<Response> {
        let headers = header.unwrap_or_default();
        let body = Self::get_data(body.unwrap_or(Body::Data(Bytes::new()))).await?;
        let req = self.build_request(method, headers, body)?;
        let res = self.client.execute(req).await?;
        Ok(res.error_for_status()?)
    }

    /// Executes a single request with a request body from a file in a directory for debugging.
    ///
    /// # Parameters
    ///
    /// * `method`: HTTP method (GET, POST, etc.) to use.
    /// * `header`: A `reqwest::header::HeaderMap` containing custom HTTP headers to be sent with
    ///   each request.
    /// * `data_dir`: Directory of files to use as request bodies.
    /// * `order`: Order to process files from the `data_dir`.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing `reqwest::Response`.
    pub async fn debug_from_dir(
        &self,
        method: HttpMethod,
        header: Option<HeaderMap>,
        data_dir: &Path,
        order: Order,
    ) -> Result<Response> {
        if method == HttpMethod::Get || method == HttpMethod::Head {
            bail!("HTTP method '{:?}' not supported", method);
        }
        let mut file_names = Self::get_file_names(data_dir).await?;
        // Sort the file names to make it deterministic.
        file_names.sort();
        let headers = header.unwrap_or_default();
        let index = match order {
            Order::Sequential => 0,
            Order::Random => rand::random_range(0..file_names.len()),
        };
        let body = fs::read(&file_names[index]).await?.into();
        let req = self.build_request(method, headers, body)?;
        let res = self.client.execute(req).await?;
        Ok(res.error_for_status()?)
    }

    /// Executes the load test with a request manifest file for debugging.
    ///
    /// This is the main method for running the test. It sends the configured number of requests
    /// concurrently to the target URL. After each request completes, it invokes the `in_progress`
    /// callback with the current, cumulative statistics.
    ///
    /// # Parameters
    ///
    /// * `method`: HTTP method (GET, POST, etc.) to use.
    /// * `manifest_file`: A manifest file.
    /// * `order`: Order to process request from the `manifest_file`.
    ///
    /// # Returns
    ///
    /// Upon completion of all requests, it returns a `Result` containing the final `LoadTestResult`
    /// with the complete summary of the test run.
    pub async fn debug_from_manifest(
        &self,
        method: HttpMethod,
        manifest_file: &Path,
        order: Order,
    ) -> Result<Response> {
        let file = File::open(manifest_file).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut templates: Vec<RequestTemplate> = Vec::new();
        while let Some(line) = lines.next_line().await? {
            let template = serde_json::from_str(&line)?;
            templates.push(template);
        }
        let index = match order {
            Order::Sequential => 0,
            Order::Random => rand::random_range(0..templates.len()),
        };
        let template = &templates[index];
        let mut headers = HeaderMap::new();
        for (name, value) in &template.headers {
            headers.insert(HeaderName::from_str(name)?, HeaderValue::from_str(value)?);
        }
        let body = if let Some(body) = &template.body {
            Bytes::from(body.clone())
        } else if let Some(base64_body) = &template.binary_body {
            Bytes::from(BASE64_STANDARD.decode(base64_body)?)
        } else {
            Bytes::new()
        };
        let req = self.build_request(method, headers, body)?;
        let res = self.client.execute(req).await?;
        Ok(res.error_for_status()?)
    }

    fn build_request(
        &self,
        method: HttpMethod,
        headers: HeaderMap,
        body: Bytes,
    ) -> Result<reqwest::Request> {
        let req_method = match method {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Patch => reqwest::Method::PATCH,
            HttpMethod::Head => reqwest::Method::HEAD,
        };
        let mut req = self.client.request(req_method, &self.url).headers(headers);
        if !body.is_empty() {
            req = req.body(body);
        }
        Ok(req.build()?)
    }

    async fn timed_request(
        &self,
        req: reqwest::Request,
        iteration: u64,
        base_file_name: Option<OsString>,
        save_response: bool,
    ) -> WorkerResult {
        let start = Instant::now();
        let res = self.client.execute(req).await;
        let res = match res {
            Ok(r) => r.error_for_status().map_err(anyhow::Error::from),
            Err(e) => Err(e.into()),
        };
        let duration = start.elapsed();
        if !save_response {
            match res {
                Ok(resp) => {
                    use futures::StreamExt;
                    let mut stream = resp.bytes_stream();
                    while let Some(chunk) = stream.next().await {
                        if chunk.is_err() {
                            return WorkerResult::StatsOnly {
                                success: false,
                                duration: start.elapsed(),
                            };
                        }
                    }
                    WorkerResult::StatsOnly {
                        success: true,
                        duration: start.elapsed(),
                    }
                }
                Err(_) => WorkerResult::StatsOnly {
                    success: false,
                    duration,
                },
            }
        } else {
            match res {
                Ok(resp) => {
                    let version = resp.version();
                    let status = resp.status();
                    let headers = resp.headers().clone();
                    match resp.bytes().await {
                        Ok(body) => WorkerResult::WithResponse {
                            res: Ok(ResponseData {
                                version,
                                status,
                                headers,
                                body,
                            }),
                            duration: start.elapsed(),
                            iteration,
                            base_file_name,
                        },
                        Err(e) => WorkerResult::WithResponse {
                            res: Err(e.into()),
                            duration: start.elapsed(),
                            iteration,
                            base_file_name,
                        },
                    }
                }
                Err(e) => WorkerResult::WithResponse {
                    res: Err(e),
                    duration,
                    iteration,
                    base_file_name,
                },
            }
        }
    }

    async fn create_identity(cert: &Path, key: &Path) -> Result<Identity> {
        if !cert.is_file() {
            bail!(
                "Certificate '{}' does not exist or is not a file",
                cert.display()
            );
        }
        if !key.is_file() {
            bail!(
                "Private key '{}' does not exist or is not a file",
                key.display()
            );
        }
        let cert_bytes = tokio::fs::read(cert).await?;
        let key_bytes = tokio::fs::read(key).await?;
        let mut pem_bytes = cert_bytes;
        pem_bytes.extend_from_slice(&key_bytes);
        Ok(Identity::from_pem(&pem_bytes)?)
    }

    async fn get_data(body: Body) -> Result<Bytes> {
        match body {
            Body::Data(data) => Ok(data),
            Body::DataFile(data_file) => {
                if !data_file.is_file() {
                    bail!(
                        "Data file '{}' does not exist or is not a file",
                        data_file.display()
                    );
                }
                let data = fs::read(data_file).await?;
                Ok(data.into())
            }
        }
    }

    async fn get_file_names(dir: &Path) -> Result<Vec<PathBuf>> {
        let mut file_names: Vec<PathBuf> = Vec::new();
        let mut read_dir = fs::read_dir(dir).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            if entry.file_type().await?.is_file() {
                file_names.push(entry.path());
            }
        }
        Ok(file_names)
    }
    async fn process_results<F>(
        &self,
        mut rx: mpsc::Receiver<WorkerResult>,
        in_progress: F,
        output_dir: Option<&Path>,
    ) -> Result<LoadTestResult>
    where
        F: Fn(&LoadTestResult),
    {
        let mut result = LoadTestResult::new();
        if let Some(output_dir) = output_dir {
            fs::create_dir_all(output_dir).await?;
        }
        let test_time = Instant::now();
        let mut last_update = Instant::now();
        let update_interval = Duration::from_millis(100);
        let output_dir = output_dir.map(|p| p.to_path_buf());
        let requests = self.requests;
        let mut file_tasks = JoinSet::new();
        while let Some(worker_res) = rx.recv().await {
            result.completed += 1;
            match worker_res {
                WorkerResult::StatsOnly { success, duration } => {
                    if success {
                        result.success += 1;
                        if self.stats == Stats::All || self.stats == Stats::Success {
                            Self::update_stats(&mut result, duration)
                        }
                    } else {
                        result.failures += 1;
                        if self.stats == Stats::All || self.stats == Stats::Error {
                            Self::update_stats(&mut result, duration)
                        }
                    }
                }
                WorkerResult::WithResponse {
                    res,
                    duration,
                    iteration,
                    base_file_name,
                } => match res {
                    Ok(response) => {
                        result.success += 1;
                        if self.stats == Stats::All || self.stats == Stats::Success {
                            Self::update_stats(&mut result, duration)
                        }
                        if let Some(output_dir) = &output_dir {
                            let output_dir = output_dir.clone();
                            let base_file_name = base_file_name.clone();
                            file_tasks.spawn(async move {
                                let output_file = Self::get_output_file(
                                    requests,
                                    &output_dir,
                                    iteration + 1,
                                    base_file_name.as_deref(),
                                    true,
                                );
                                let _ = Self::write_success_output_file(
                                    output_file,
                                    response,
                                    duration,
                                )
                                .await;
                            });
                        }
                    }
                    Err(error) => {
                        result.failures += 1;
                        if self.stats == Stats::All || self.stats == Stats::Error {
                            Self::update_stats(&mut result, duration)
                        }
                        if let Some(output_dir) = &output_dir {
                            let output_dir = output_dir.clone();
                            let base_file_name = base_file_name.clone();
                            file_tasks.spawn(async move {
                                let output_file = Self::get_output_file(
                                    requests,
                                    &output_dir,
                                    iteration + 1,
                                    base_file_name.as_deref(),
                                    false,
                                );
                                let _ = Self::write_failure_output_file(output_file, error).await;
                            });
                        }
                    }
                },
            }
            if last_update.elapsed() >= update_interval {
                result.elapsed = test_time.elapsed();
                let elapsed_secs = result.elapsed.as_secs_f64();
                if elapsed_secs > 0.0 {
                    result.rps = result.completed as f64 / elapsed_secs;
                }
                if result.completed > 0 {
                    result.success_rate = (result.success as f64 / result.completed as f64) * 100.0;
                    result.failure_rate =
                        (result.failures as f64 / result.completed as f64) * 100.0;
                }
                result.update_percentiles();
                in_progress(&result);
                last_update = Instant::now();
            }
        }
        result.elapsed = test_time.elapsed();
        let elapsed_secs = result.elapsed.as_secs_f64();
        if elapsed_secs > 0.0 {
            result.rps = result.completed as f64 / elapsed_secs;
        }
        if result.completed > 0 {
            result.success_rate = (result.success as f64 / result.completed as f64) * 100.0;
            result.failure_rate = (result.failures as f64 / result.completed as f64) * 100.0;
        }
        result.update_percentiles();
        in_progress(&result);
        // Wait for all file writing tasks to complete.
        while file_tasks.join_next().await.is_some() {}
        result.elapsed = test_time.elapsed();
        let elapsed_secs = result.elapsed.as_secs_f64();
        if elapsed_secs > 0.0 {
            result.rps = result.completed as f64 / elapsed_secs;
        }
        if result.completed > 0 {
            result.success_rate = (result.success as f64 / result.completed as f64) * 100.0;
            result.failure_rate = (result.failures as f64 / result.completed as f64) * 100.0;
        }
        Ok(result)
    }

    fn update_stats(result: &mut LoadTestResult, duration: Duration) {
        let _ = result.durations.record(duration.as_micros() as u64);
        result.total_duration += duration;
        result.min = if result.min == Duration::default() {
            duration
        } else {
            result.min.min(duration)
        };
        result.max = result.max.max(duration);
    }

    fn get_output_file(
        num_requests: u32,
        output_dir: &Path,
        iteration: u64,
        base_file_name: Option<&OsStr>,
        success: bool,
    ) -> PathBuf {
        if let Some(base_file_name) = base_file_name {
            output_dir.join(format!(
                "{}-{:0width$}-{}.json",
                if success { "success" } else { "failure" },
                iteration,
                base_file_name.to_string_lossy(),
                width = num_requests.to_string().len()
            ))
        } else {
            output_dir.join(format!(
                "{}-{:0width$}.json",
                if success { "success" } else { "failure" },
                iteration,
                width = num_requests.to_string().len()
            ))
        }
    }

    async fn write_success_output_file(
        output_file: PathBuf,
        response: ResponseData,
        duration: Duration,
    ) -> Result<()> {
        let version: String = format!("{:?}", response.version);
        let status_code = response.status.as_u16();
        let headers: HashMap<String, String> = response
            .headers
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
            .collect();
        let body_bytes = response.body;
        let body_string: String = match str::from_utf8(&body_bytes) {
            Ok(bytes) => bytes.to_string(),
            Err(_) => BASE64_STANDARD.encode(&body_bytes),
        };
        let output = json!({
            "version": version,
            "status": status_code,
            "headers": headers,
            "body": body_string,
            "duration": duration,
        });
        Ok(fs::write(output_file, serde_json::to_string_pretty(&output)?).await?)
    }

    async fn write_failure_output_file(output_file: PathBuf, error: anyhow::Error) -> Result<()> {
        let output = json!({
            "error": error.to_string(),
        });
        Ok(fs::write(output_file, serde_json::to_string_pretty(&output)?).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_succeeds() {
        let result = LoadTestRunner::new(
            "http://localhost:8080",
            10,
            2,
            Stats::Success,
            None,
            None,
            None,
            false,
            None,
            None,
        )
        .await
        .unwrap();

        assert_eq!(result.url, "http://localhost:8080");
        assert_eq!(result.requests, 10);
        assert_eq!(result.concurrency, 2);
    }

    #[tokio::test]
    async fn new_url_is_empty_fails() {
        let result = LoadTestRunner::new(
            "",
            2,
            2,
            Stats::Success,
            None,
            None,
            None,
            false,
            None,
            None,
        )
        .await
        .unwrap_err();

        assert_eq!(result.to_string(), "URL cannot be empty");
    }

    #[tokio::test]
    async fn new_num_requests_is_zero_fails() {
        let result = LoadTestRunner::new(
            "http://localhost:8080",
            0,
            2,
            Stats::Success,
            None,
            None,
            None,
            false,
            None,
            None,
        )
        .await
        .unwrap_err();

        assert_eq!(result.to_string(), "Number of requests cannot be zero");
    }

    #[tokio::test]
    async fn new_num_concurrency_is_zero_fails() {
        let result = LoadTestRunner::new(
            "http://localhost:8080",
            2,
            0,
            Stats::Success,
            None,
            None,
            None,
            false,
            None,
            None,
        )
        .await
        .unwrap_err();

        assert_eq!(result.to_string(), "Number of concurrency cannot be zero");
    }

    #[tokio::test]
    async fn new_num_concurrency_greater_than_num_requests_fails() {
        let result = LoadTestRunner::new(
            "http://localhost:8080",
            2,
            3,
            Stats::Success,
            None,
            None,
            None,
            false,
            None,
            None,
        )
        .await
        .unwrap_err();

        assert_eq!(
            result.to_string(),
            "Number of concurrency: 3 must be less than or equal to number of requests: 2"
        );
    }

    #[tokio::test]
    async fn new_ca_cert_does_not_exist_fails() {
        let result = LoadTestRunner::new(
            "http://localhost:8080",
            10,
            2,
            Stats::Success,
            Some(Path::new("doesnotexist")),
            None,
            None,
            false,
            None,
            None,
        )
        .await
        .unwrap_err();

        assert_eq!(
            result.to_string(),
            "CA certificate 'doesnotexist' does not exist or is not a file"
        );
    }

    #[tokio::test]
    async fn new_cert_does_not_exist_fails() {
        let result = LoadTestRunner::new(
            "http://localhost:8080",
            10,
            2,
            Stats::Success,
            None,
            Some(Path::new("doesnotexist")),
            Some(Path::new("tests/tls/key.pem")),
            false,
            None,
            None,
        )
        .await
        .unwrap_err();

        assert_eq!(
            result.to_string(),
            "Certificate 'doesnotexist' does not exist or is not a file"
        );
    }

    #[tokio::test]
    async fn new_key_does_not_exist_fails() {
        let result = LoadTestRunner::new(
            "http://localhost:8080",
            10,
            2,
            Stats::Success,
            None,
            Some(Path::new("tests/tls/client.crt")),
            Some(Path::new("doesnotexist")),
            false,
            None,
            None,
        )
        .await
        .unwrap_err();

        assert_eq!(
            result.to_string(),
            "Private key 'doesnotexist' does not exist or is not a file"
        );
    }

    #[tokio::test]
    async fn get_file_names_succeeds() {
        let mut file_names = LoadTestRunner::get_file_names(Path::new("tests/test_requests"))
            .await
            .unwrap();
        file_names.sort();

        assert_eq!(
            file_names,
            vec![
                PathBuf::from("tests/test_requests/test1.json"),
                PathBuf::from("tests/test_requests/test2.json"),
                PathBuf::from("tests/test_requests/test3.json"),
                PathBuf::from("tests/test_requests/test4.json"),
                PathBuf::from("tests/test_requests/test5.json"),
            ]
        );
    }

    #[tokio::test]
    async fn get_data_succeeds() {
        let bytes = LoadTestRunner::get_data(Body::Data("Hello".into()))
            .await
            .unwrap();

        assert_eq!(bytes, "Hello".as_bytes());
    }

    #[tokio::test]
    async fn get_data_file_succeeds() {
        let bytes = LoadTestRunner::get_data(Body::DataFile(PathBuf::from(
            "tests/test_requests/test1.json",
        )))
        .await
        .unwrap();

        assert_eq!(bytes, "{\n  \"message\": \"hello1\"\n}\n".as_bytes());
    }

    #[tokio::test]
    async fn get_data_file_does_not_exist_fails() {
        let err = LoadTestRunner::get_data(Body::DataFile(PathBuf::from("doesnotexist")))
            .await
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            "Data file 'doesnotexist' does not exist or is not a file"
        );
    }

    #[tokio::test]
    async fn get_invalid_data_file_fails() {
        let err = LoadTestRunner::get_data(Body::DataFile(PathBuf::from("tests/test_requests")))
            .await
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            "Data file 'tests/test_requests' does not exist or is not a file"
        );
    }

    #[test]
    fn get_output_file_succeeds() {
        let output_file = LoadTestRunner::get_output_file(100, Path::new("/tmp"), 3, None, true);
        assert_eq!(output_file, Path::new("/tmp/success-003.json"));

        let output_file = LoadTestRunner::get_output_file(100, Path::new("/tmp"), 3, None, false);
        assert_eq!(output_file, Path::new("/tmp/failure-003.json"));

        let output_file = LoadTestRunner::get_output_file(
            100,
            Path::new("/tmp"),
            3,
            Some(OsStr::new("request")),
            true,
        );
        assert_eq!(output_file, Path::new("/tmp/success-003-request.json"));

        let output_file = LoadTestRunner::get_output_file(
            100,
            Path::new("/tmp"),
            3,
            Some(OsStr::new("request")),
            false,
        );
        assert_eq!(output_file, Path::new("/tmp/failure-003-request.json"));
    }
}
