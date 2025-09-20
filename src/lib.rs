use anyhow::{Result, bail};
use bytes::Bytes;
use futures::{Stream, StreamExt, stream};
use rand::Rng;
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
#[derive(Debug, Clone)]
pub enum Order {
    /// Process files in alphabetical order (default).
    Sequential,

    /// Process files in a random order.
    Random,
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
        insecure: &Option<bool>,
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
                "Number of concurrency: {concurrency} must be less than number of requests: {requests}"
            );
        }
        let mut builder = Client::builder()
            .use_rustls_tls()
            .danger_accept_invalid_certs(insecure.unwrap_or(false));
        if let Some(ca_cert_path) = ca_cert {
            if !ca_cert_path.is_file() {
                bail!(
                    "CA certificate '{}' does not exist or is not a file",
                    ca_cert_path.to_str().unwrap()
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
        in_progress: T,
    ) -> Result<LoadTestResult>
    where
        T: Fn(&LoadTestResult),
    {
        let body = Self::get_data(&body.unwrap_or(Body::Data(Bytes::new()))).await?;
        let stream = stream::iter(0..self.requests as u64)
            .map(|_| {
                let header = header.clone().unwrap_or_default();
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
        self.process_stream(stream, in_progress).await
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
        data_dir: &PathBuf,
        order: Order,
        in_progress: T,
    ) -> Result<LoadTestResult>
    where
        T: Fn(&LoadTestResult),
    {
        if method == HttpMethod::Get || method == HttpMethod::Head {
            bail!("HTTP method '{:?}' not supported", method);
        }
        let mut filenames = Self::get_filenames(data_dir).await?;
        // Sort the file names to make it deterministic.
        filenames.sort();
        let mut random = rand::rng();
        let stream = stream::iter(0..self.requests as u64)
            .map(|i| {
                let header = header.clone().unwrap_or_default();
                let index = match order {
                    Order::Sequential => i as usize % filenames.len(),
                    Order::Random => random.random_range(0..filenames.len()),
                };
                let path = &filenames[index];
                async move {
                    let body = match fs::read(path).await {
                        Ok(data) => data.into(),
                        Err(e) => return (Err(e.into()), Duration::default()),
                    };
                    let start_time = Instant::now();
                    let response = match method {
                        HttpMethod::Post => self.post(header, body).await,
                        HttpMethod::Put => self.put(header, body).await,
                        HttpMethod::Delete => self.delete(header, body).await,
                        HttpMethod::Patch => self.patch(header, body).await,
                        _ => panic!("Unexpected HTTP method '{method:?}'"),
                    };
                    let duration = start_time.elapsed();
                    (response, duration)
                }
            })
            .buffer_unordered(self.concurrency as usize);
        self.process_stream(stream, in_progress).await
    }

    async fn create_identity(cert: &PathBuf, key: &PathBuf) -> Result<Identity> {
        if !cert.is_file() {
            bail!(
                "Certificate '{}' does not exist or is not a file",
                cert.to_str().unwrap()
            );
        }
        if !key.is_file() {
            bail!(
                "Private key '{}' does not exist or is not a file",
                key.to_str().unwrap()
            );
        }
        let cert_bytes = tokio::fs::read(cert).await?;
        let key_bytes = tokio::fs::read(key).await?;
        let mut pem_bytes = cert_bytes;
        pem_bytes.extend_from_slice(&key_bytes);
        Ok(Identity::from_pem(&pem_bytes)?)
    }

    async fn get_data(body: &Body) -> Result<Bytes> {
        match body {
            Body::Data(data) => Ok(data.clone()),
            Body::DataFile(data_file) => {
                if !data_file.is_file() {
                    bail!(
                        "Data file '{}' does not exist or is not a file",
                        data_file.to_str().unwrap()
                    );
                }
                let data = fs::read(data_file).await?;
                Ok(data.into())
            }
        }
    }

    async fn get_filenames(dir: &PathBuf) -> Result<Vec<PathBuf>> {
        let mut filenames: Vec<PathBuf> = Vec::new();
        let mut read_dir = fs::read_dir(dir).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            if entry.file_type().await?.is_file() {
                filenames.push(entry.path());
            }
        }
        Ok(filenames)
    }

    async fn process_stream<S, F>(&self, mut stream: S, in_progress: F) -> Result<LoadTestResult>
    where
        S: Stream<Item = (Result<Response, anyhow::Error>, Duration)> + Unpin,
        F: Fn(&LoadTestResult),
    {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_succeeds() {
        let result =
            LoadTestRunner::new("http://localhost:8080", 10, 2, &None, &None, &None, &None)
                .await
                .unwrap();

        assert_eq!(result.url, "http://localhost:8080");
        assert_eq!(result.requests, 10);
        assert_eq!(result.concurrency, 2);
    }

    #[tokio::test]
    async fn new_url_is_empty_fails() {
        let result = LoadTestRunner::new("", 2, 2, &None, &None, &None, &None)
            .await
            .unwrap_err();

        assert_eq!(result.to_string(), "URL cannot be empty");
    }

    #[tokio::test]
    async fn new_num_requests_is_zero_fails() {
        let result = LoadTestRunner::new("http://localhost:8080", 0, 2, &None, &None, &None, &None)
            .await
            .unwrap_err();

        assert_eq!(result.to_string(), "Number of requests cannot be zero");
    }

    #[tokio::test]
    async fn new_num_concurrency_is_zero_fails() {
        let result = LoadTestRunner::new("http://localhost:8080", 2, 0, &None, &None, &None, &None)
            .await
            .unwrap_err();

        assert_eq!(result.to_string(), "Number of concurrency cannot be zero");
    }

    #[tokio::test]
    async fn new_num_concurrency_greater_than_num_requests_fails() {
        let result = LoadTestRunner::new("http://localhost:8080", 2, 3, &None, &None, &None, &None)
            .await
            .unwrap_err();

        assert_eq!(
            result.to_string(),
            "Number of concurrency: 3 must be less than number of requests: 2"
        );
    }

    #[tokio::test]
    async fn new_ca_cert_does_not_exist_fails() {
        let result = LoadTestRunner::new(
            "http://localhost:8080",
            10,
            2,
            &Some("doesnotexist".into()),
            &None,
            &None,
            &None,
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
            &None,
            &Some("doesnotexist".into()),
            &Some("tests/tls/key.pem".into()),
            &None,
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
            &None,
            &Some("tests/tls/cert.pem".into()),
            &Some("doesnotexist".into()),
            &None,
        )
        .await
        .unwrap_err();

        assert_eq!(
            result.to_string(),
            "Private key 'doesnotexist' does not exist or is not a file"
        );
    }

    #[tokio::test]
    async fn get_filenames_succeeds() {
        let mut filenames = LoadTestRunner::get_filenames(&"tests/test_requests".into())
            .await
            .unwrap();
        filenames.sort();

        assert_eq!(
            filenames,
            vec![
                PathBuf::from("tests/test_requests/test1.json"),
                "tests/test_requests/test2.json".into(),
                "tests/test_requests/test3.json".into(),
                "tests/test_requests/test4.json".into(),
                "tests/test_requests/test5.json".into(),
            ]
        );
    }

    #[tokio::test]
    async fn get_data_succeeds() {
        let bytes = LoadTestRunner::get_data(&Body::Data("Hello".into()))
            .await
            .unwrap();

        assert_eq!(bytes, "Hello".as_bytes());
    }

    #[tokio::test]
    async fn get_data_file_succeeds() {
        let bytes =
            LoadTestRunner::get_data(&Body::DataFile("tests/test_requests/test1.json".into()))
                .await
                .unwrap();

        assert_eq!(bytes, "{\n  \"message\": \"hello1\"\n}\n".as_bytes());
    }

    #[tokio::test]
    async fn get_data_file_does_not_exist_fails() {
        let err = LoadTestRunner::get_data(&Body::DataFile("doesnotexist".into()))
            .await
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            "Data file 'doesnotexist' does not exist or is not a file"
        );
    }

    #[tokio::test]
    async fn get_invalid_data_file_fails() {
        let err = LoadTestRunner::get_data(&Body::DataFile("tests/test_requests".into()))
            .await
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            "Data file 'tests/test_requests' does not exist or is not a file"
        );
    }
}
