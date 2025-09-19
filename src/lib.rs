use anyhow::Result;
use bytes::Bytes;
use futures::{StreamExt, stream};
use reqwest::header::HeaderMap;
use reqwest::{Certificate, Client, Identity, Response};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::fs;

/// A load test runner responsible for configuring and executing a load test.
#[derive(Debug, Clone)]
pub struct LoadTestRunner {
    /// Target URL to send requests to.
    pub url: String,

    /// Total number of requests to send.
    pub requests: u32,

    /// Number of concurrent requests to run at a time.
    pub concurrency: u32,

    /// HTTP client.
    client: Client,
}

/// Defines the allowed HTTP methods that the user can specify.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

    /// A collection of individual response durations for each successful request.
    /// This is used to calculate percentiles.
    pub durations: Vec<Duration>,

    /// The average response time for successful requests.
    pub avg: Duration,

    /// The 50th percentile (median) response time for successful requests.
    pub p50: Duration,

    /// The 90th percentile response time for successful requests.
    pub p90: Duration,

    /// The 95th percentile response time for successful requests.
    pub p95: Duration,
}

impl LoadTestResult {
    fn new() -> Self {
        LoadTestResult {
            success: 0,
            failures: 0,
            completed: 0,
            total_duration: Duration::default(),
            durations: Vec::new(),
            avg: Duration::default(),
            p50: Duration::default(),
            p90: Duration::default(),
            p95: Duration::default(),
        }
    }
}

impl LoadTestRunner {
    /// Creates a new `LoadTestRunner` with the specified configuration.
    ///
    /// # Parameters
    ///
    /// * `url`: Target URL to send requests to.
    /// * `requests`: Total number of requests to send.
    /// * `concurrency`: Number of concurrent requests to run at a time.
    /// * `ca_cert`: Custom CA certificate file (PEM format).
    /// * `cert`: Public certificate file (PEM format).
    /// * `key`: Private key file (PEM format).
    ///
    /// # Returns
    /// A `Result` containing the new `LoadTestRunner` instance if successful.
    pub async fn new(
        url: &str,
        requests: u32,
        concurrency: u32,
        ca_cert: &Option<PathBuf>,
        cert: &Option<PathBuf>,
        key: &Option<PathBuf>,
    ) -> Result<Self> {
        let mut builder = Client::builder().use_rustls_tls();
        if let Some(ca_cert_path) = ca_cert {
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
        header: HeaderMap,
        body: Bytes,
        in_progress: T,
    ) -> Result<LoadTestResult>
    where
        T: Fn(&LoadTestResult),
    {
        let mut stream = stream::iter(0..self.requests as u64)
            .map(|_| {
                let header = header.clone();
                let body = body.clone();
                async move {
                    let start_time = Instant::now();
                    let response = match method {
                        HttpMethod::Get => self.get(header).await,
                        HttpMethod::Post => self.post(header, body).await,
                        HttpMethod::Put => self.put(header, body).await,
                        HttpMethod::Delete => self.delete(header, body).await,
                        HttpMethod::Patch => self.patch(header, body).await,
                        HttpMethod::Head => self.head(header).await,
                    };
                    let duration = start_time.elapsed();
                    (response, duration)
                }
            })
            .buffer_unordered(self.concurrency as usize);

        let mut result = LoadTestResult::new();
        while let Some((res, duration)) = stream.next().await {
            result.completed += 1;
            match res {
                Ok(_) => {
                    result.success += 1;
                    // Only capture the duration for successful request.
                    result.total_duration += duration;
                    result.avg = result.total_duration / result.completed;
                    result.durations.push(duration);
                }
                Err(_) => {
                    result.failures += 1;
                }
            }
            in_progress(&result);
        }

        let (p50, p90, p95) = if !result.durations.is_empty() {
            result.durations.sort();
            let p50_index = (result.durations.len() as f64 * 0.50) as usize;
            let p90_index = (result.durations.len() as f64 * 0.90) as usize;
            let p95_index = (result.durations.len() as f64 * 0.95) as usize;
            let p50_val = result.durations.get(p50_index).cloned().unwrap_or_default();
            let p90_val = result.durations.get(p90_index).cloned().unwrap_or_default();
            let p95_val = result.durations.get(p95_index).cloned().unwrap_or_default();
            (p50_val, p90_val, p95_val)
        } else {
            (
                Duration::default(),
                Duration::default(),
                Duration::default(),
            )
        };
        result.p50 = p50;
        result.p90 = p90;
        result.p95 = p95;
        result.avg = if self.requests > 0 {
            result.total_duration / self.requests
        } else {
            Duration::new(0, 0)
        };

        Ok(result)
    }

    async fn create_identity(cert: &PathBuf, key: &PathBuf) -> Result<Identity> {
        let cert_bytes = tokio::fs::read(cert).await?;
        let key_bytes = tokio::fs::read(key).await?;
        let mut pem_bytes = cert_bytes;
        pem_bytes.extend_from_slice(&key_bytes);
        Ok(Identity::from_pem(&pem_bytes)?)
    }

    async fn get(&self, headers: HeaderMap) -> Result<Response> {
        Ok(self
            .client
            .get(&self.url)
            .headers(headers)
            .send()
            .await?
            .error_for_status()?)
    }

    async fn post(&self, headers: HeaderMap, body: Bytes) -> Result<Response> {
        Ok(self
            .client
            .post(&self.url)
            .headers(headers)
            .body(body.clone())
            .send()
            .await?
            .error_for_status()?)
    }

    async fn put(&self, headers: HeaderMap, body: Bytes) -> Result<Response> {
        Ok(self
            .client
            .put(&self.url)
            .headers(headers)
            .body(body.clone())
            .send()
            .await?
            .error_for_status()?)
    }

    async fn delete(&self, headers: HeaderMap, body: Bytes) -> Result<Response> {
        Ok(self
            .client
            .delete(&self.url)
            .headers(headers)
            .body(body.clone())
            .send()
            .await?
            .error_for_status()?)
    }

    async fn patch(&self, headers: HeaderMap, body: Bytes) -> Result<Response> {
        Ok(self
            .client
            .patch(&self.url)
            .headers(headers)
            .body(body.clone())
            .send()
            .await?
            .error_for_status()?)
    }

    async fn head(&self, headers: HeaderMap) -> Result<Response> {
        Ok(self
            .client
            .head(&self.url)
            .headers(headers)
            .send()
            .await?
            .error_for_status()?)
    }
}
