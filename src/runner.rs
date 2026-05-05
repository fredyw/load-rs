use crate::generator::RequestGenerator;
use crate::models::{
    Body, HttpMethod, LoadTestEvent, LoadTestResult, Order, RequestResult, ResponseData, Stats,
};

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
    /// Quiet mode: suppress progress updates.
    pub quiet: bool,
    /// HTTP client.
    client: Client,
}

/// A builder for creating a `LoadTestRunner`.
pub struct LoadTestRunnerBuilder {
    config: crate::models::LoadTestConfig,
}

impl LoadTestRunnerBuilder {
    /// Creates a new `LoadTestRunnerBuilder` with the specified mandatory parameters.
    pub fn new(url: impl Into<String>, requests: u32, concurrency: u32) -> Self {
        Self {
            config: crate::models::LoadTestConfig {
                url: url.into(),
                requests,
                concurrency,
                stats: Stats::All,
                ca_cert: None,
                cert: None,
                key: None,
                insecure: false,
                timeout: None,
                user_agent: None,
                proxy: None,
                quiet: false,
            },
        }
    }

    /// Sets whether to suppress progress updates.
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.config.quiet = quiet;
        self
    }

    /// Sets which requests to include in the statistics.
    pub fn stats(mut self, stats: Stats) -> Self {
        self.config.stats = stats;
        self
    }

    /// Sets a custom CA certificate file (PEM format).
    pub fn ca_cert(mut self, ca_cert: impl Into<PathBuf>) -> Self {
        self.config.ca_cert = Some(ca_cert.into());
        self
    }

    /// Sets a public certificate file (PEM format).
    pub fn cert(mut self, cert: impl Into<PathBuf>) -> Self {
        self.config.cert = Some(cert.into());
        self
    }

    /// Sets a private key file (PEM format).
    pub fn key(mut self, key: impl Into<PathBuf>) -> Self {
        self.config.key = Some(key.into());
        self
    }

    /// Sets whether to allow insecure connections by skipping TLS certificate verification.
    pub fn insecure(mut self, insecure: bool) -> Self {
        self.config.insecure = insecure;
        self
    }

    /// Sets a request timeout in seconds.
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.config.timeout = Some(timeout);
        self
    }

    /// Sets a custom user agent.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = Some(user_agent.into());
        self
    }

    /// Sets a proxy server URL.
    pub fn proxy(mut self, proxy: impl Into<String>) -> Self {
        self.config.proxy = Some(proxy.into());
        self
    }

    /// Builds the `LoadTestRunner`.
    pub async fn build(self) -> Result<LoadTestRunner> {
        LoadTestRunner::new(self.config).await
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RequestTemplate {
    method: Option<HttpMethod>,
    path: Option<String>,
    #[serde(default)]
    headers: HashMap<String, String>,
    body: Option<String>,
    binary_body: Option<String>,
}

enum WorkerResult {
    StatsOnly {
        success: bool,
        duration: Duration,
        iteration: u64,
        status_code: Option<reqwest::StatusCode>,
        error: Option<String>,
    },
    WithResponse {
        res: Result<ResponseData>,
        duration: Duration,
        iteration: u64,
        base_file_name: Option<OsString>,
    },
}

impl LoadTestRunner {
    /// Creates a new `LoadTestRunnerBuilder` with the specified mandatory parameters.
    pub fn builder(
        url: impl Into<String>,
        requests: u32,
        concurrency: u32,
    ) -> LoadTestRunnerBuilder {
        LoadTestRunnerBuilder::new(url, requests, concurrency)
    }

    /// Creates a new `LoadTestRunner` with the specified configuration.
    ///
    /// # Parameters
    ///
    /// * `config`: Configuration for the load test run.
    ///
    /// # Returns
    /// A `Result` containing the new `LoadTestRunner` instance if successful.
    pub async fn new(config: crate::models::LoadTestConfig) -> Result<Self> {
        if config.url.is_empty() {
            bail!("URL cannot be empty");
        }
        if config.requests == 0 {
            bail!("Number of requests cannot be zero");
        }
        if config.concurrency == 0 {
            bail!("Number of concurrency cannot be zero");
        }
        if config.concurrency > config.requests {
            bail!(
                "Number of concurrency: {} must be less than or equal to number of requests: {}",
                config.concurrency,
                config.requests
            );
        }
        let mut builder = Client::builder()
            .use_rustls_tls()
            .danger_accept_invalid_certs(config.insecure)
            .tcp_nodelay(true)
            .pool_max_idle_per_host(config.concurrency as usize);
        if let Some(t) = config.timeout {
            builder = builder.timeout(Duration::from_secs(t));
        }
        if let Some(ua) = config.user_agent {
            builder = builder.user_agent(ua);
        }
        if let Some(p) = config.proxy {
            builder = builder.proxy(reqwest::Proxy::all(p)?);
        }
        if let Some(ca_cert_path) = config.ca_cert {
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
        if let (Some(cert_path), Some(key_path)) = (config.cert, config.key) {
            builder = builder.identity(Self::create_identity(&cert_path, &key_path).await?);
        }
        Ok(LoadTestRunner {
            url: config.url,
            requests: config.requests,
            concurrency: config.concurrency,
            stats: config.stats,
            quiet: config.quiet,
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
    /// Executes the load test using a custom request generator.
    ///
    /// This method allows for maximum flexibility by providing a `RequestGenerator`
    /// which can dynamically create requests for each iteration.
    ///
    /// # Parameters
    ///
    /// * `generator`: A request generator that implements the `RequestGenerator` trait.
    /// * `output_dir`: Optional directory to save response data.
    /// * `in_progress`: A callback function that is invoked as the test progresses.
    ///
    /// # Returns
    /// Upon completion of all requests, it returns a `Result` containing the final `LoadTestResult`.
    pub async fn run_with_generator<G, T>(
        &self,
        generator: G,
        output_dir: Option<&Path>,
        in_progress: T,
    ) -> Result<LoadTestResult>
    where
        G: RequestGenerator + 'static,
        T: Fn(LoadTestEvent),
    {
        let generator = Arc::new(generator);
        let save_response = output_dir.is_some();
        let (tx, rx) = mpsc::channel(self.concurrency as usize);
        let runner = Arc::new(self.clone());
        let counter = Arc::new(AtomicU64::new(0));
        for _ in 0..self.concurrency {
            let tx = tx.clone();
            let runner = Arc::clone(&runner);
            let counter = Arc::clone(&counter);
            let generator = Arc::clone(&generator);
            tokio::spawn(async move {
                loop {
                    let i = counter.fetch_add(1, Ordering::Relaxed);
                    if i >= runner.requests as u64 {
                        break;
                    }
                    let req_gen_result = generator.generate(i);
                    let result = match req_gen_result {
                        Ok((req, base_file_name)) => {
                            runner
                                .timed_request(req, i, base_file_name, save_response)
                                .await
                        }
                        Err(e) => WorkerResult::StatsOnly {
                            success: false,
                            duration: Duration::default(),
                            iteration: i,
                            status_code: None,
                            error: Some(e.to_string()),
                        },
                    };
                    if tx.send(result).await.is_err() {
                        break;
                    }
                }
            });
        }
        drop(tx);
        self.process_results(rx, in_progress, output_dir).await
    }

    /// Executes the load test with the specified HTTP method, headers, and body.
    ///
    /// # Parameters
    ///
    /// * `method`: HTTP method (GET, POST, etc.) to use.
    /// * `header`: Optional custom HTTP headers.
    /// * `body`: Optional request body.
    /// * `output_dir`: Optional directory to save response data.
    /// * `in_progress`: A callback function that is invoked as the test progresses.
    ///
    /// # Returns
    /// Upon completion of all requests, it returns a `Result` containing the final `LoadTestResult`.
    pub async fn run<T>(
        &self,
        method: HttpMethod,
        header: Option<HeaderMap>,
        body: Option<Body>,
        output_dir: Option<&Path>,
        in_progress: T,
    ) -> Result<LoadTestResult>
    where
        T: Fn(LoadTestEvent),
    {
        let body = Self::get_data(body.unwrap_or(Body::Data(Bytes::new()))).await?;
        let headers = header.unwrap_or_default();
        let req = self.build_request(method, &self.url, headers, body)?;
        let req = Arc::new(req);
        self.run_with_generator(
            move |_| Ok(((*req).try_clone().unwrap(), None)),
            output_dir,
            in_progress,
        )
        .await
    }

    fn join_url(base: &str, path: Option<&str>) -> Result<String> {
        if let Some(path) = path {
            let mut url = reqwest::Url::parse(base)?;
            if path.starts_with('/') {
                url.set_path(path);
            } else {
                let mut current_path = url.path().to_string();
                if !current_path.ends_with('/') {
                    current_path.push('/');
                }
                current_path.push_str(path);
                url.set_path(&current_path);
            }
            Ok(url.to_string())
        } else {
            Ok(base.to_string())
        }
    }

    /// Executes the load test using request bodies from a directory.
    ///
    /// # Parameters
    ///
    /// * `method`: HTTP method (POST, PUT, etc.) to use.
    /// * `header`: Optional custom HTTP headers.
    /// * `data_dir`: Directory containing files to be used as request bodies.
    /// * `order`: Order in which to process files from the directory.
    /// * `output_dir`: Optional directory to save response data.
    /// * `in_progress`: A callback function that is invoked as the test progresses.
    ///
    /// # Returns
    /// Upon completion of all requests, it returns a `Result` containing the final `LoadTestResult`.
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
        T: Fn(LoadTestEvent),
    {
        if method == HttpMethod::Get || method == HttpMethod::Head {
            bail!("HTTP method '{:?}' not supported", method);
        }
        let mut file_names = Self::get_file_names(data_dir).await?;
        if file_names.is_empty() {
            bail!("No files found in directory '{}'", data_dir.display());
        }
        file_names.sort();
        let mut bodies = Vec::new();
        for path in &file_names {
            let body = tokio::fs::read(path).await?;
            bodies.push((
                Arc::new(Bytes::from(body)),
                path.file_stem().map(|f| f.to_owned()),
            ));
        }
        let mut reqs = Vec::new();
        for (body, base_file_name) in bodies.iter() {
            let req = self.build_request(
                method,
                &self.url,
                header.clone().unwrap_or_default(),
                (**body).clone(),
            )?;
            reqs.push((req, base_file_name.clone()));
        }
        let reqs = Arc::new(reqs);
        self.run_with_generator(
            move |iteration| {
                let index = match order {
                    Order::Sequential => iteration as usize % reqs.len(),
                    Order::Random => rand::random_range(0..reqs.len()),
                };
                let (req, base_file_name) = &reqs[index];
                Ok((req.try_clone().unwrap(), base_file_name.clone()))
            },
            output_dir,
            in_progress,
        )
        .await
    }

    /// Executes the load test using a request manifest file (JSON Lines format).
    ///
    /// # Parameters
    ///
    /// * `method`: HTTP method (POST, PUT, etc.) to use.
    /// * `manifest_file`: Path to the manifest file.
    /// * `order`: Order in which to process requests from the manifest.
    /// * `output_dir`: Optional directory to save response data.
    /// * `in_progress`: A callback function that is invoked as the test progresses.
    ///
    /// # Returns
    /// Upon completion of all requests, it returns a `Result` containing the final `LoadTestResult`.
    pub async fn run_from_manifest<T>(
        &self,
        method: HttpMethod,
        manifest_file: &Path,
        order: Order,
        output_dir: Option<&Path>,
        in_progress: T,
    ) -> Result<LoadTestResult>
    where
        T: Fn(LoadTestEvent),
    {
        let file = tokio::fs::File::open(manifest_file).await?;
        let reader = tokio::io::BufReader::new(file);
        let mut lines = tokio::io::AsyncBufReadExt::lines(reader);
        let mut reqs = Vec::new();
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
            let req_method = template.method.unwrap_or(method);
            let req_url = Self::join_url(&self.url, template.path.as_deref())?;
            let req = self.build_request(req_method, &req_url, headers, body)?;
            reqs.push(req);
        }
        if reqs.is_empty() {
            bail!(
                "No requests found in manifest file '{}'",
                manifest_file.display()
            );
        }
        let reqs = Arc::new(reqs);
        self.run_with_generator(
            move |iteration| {
                let index = match order {
                    Order::Sequential => iteration as usize % reqs.len(),
                    Order::Random => rand::random_range(0..reqs.len()),
                };
                let req = &reqs[index];
                Ok((req.try_clone().unwrap(), None))
            },
            output_dir,
            in_progress,
        )
        .await
    }

    /// Executes a single request for debugging purposes.
    ///
    /// # Parameters
    ///
    /// * `method`: HTTP method to use.
    /// * `header`: Optional custom HTTP headers.
    /// * `body`: Optional request body.
    ///
    /// # Returns
    /// Returns a `Result` containing the `reqwest::Response`.
    pub async fn debug(
        &self,
        method: HttpMethod,
        header: Option<HeaderMap>,
        body: Option<Body>,
    ) -> Result<Response> {
        let headers = header.unwrap_or_default();
        let body = Self::get_data(body.unwrap_or(Body::Data(Bytes::new()))).await?;
        let req = self.build_request(method, &self.url, headers, body)?;
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
        if file_names.is_empty() {
            bail!("No files found in directory '{}'", data_dir.display());
        }
        // Sort the file names to make it deterministic.
        file_names.sort();
        let headers = header.unwrap_or_default();
        let index = match order {
            Order::Sequential => 0,
            Order::Random => rand::random_range(0..file_names.len()),
        };
        let body = fs::read(&file_names[index]).await?.into();
        let req = self.build_request(method, &self.url, headers, body)?;
        let res = self.client.execute(req).await?;
        Ok(res.error_for_status()?)
    }

    /// Executes a single request from a manifest file for debugging.
    ///
    /// # Parameters
    ///
    /// * `method`: HTTP method to use.
    /// * `manifest_file`: Path to the manifest file.
    /// * `order`: Order to select a request from the manifest.
    ///
    /// # Returns
    /// Returns a `Result` containing the `reqwest::Response`.
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
        if templates.is_empty() {
            bail!(
                "No requests found in manifest file '{}'",
                manifest_file.display()
            );
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
        let req_method = template.method.unwrap_or(method);
        let req_url = Self::join_url(&self.url, template.path.as_deref())?;
        let req = self.build_request(req_method, &req_url, headers, body)?;
        let res = self.client.execute(req).await?;
        Ok(res.error_for_status()?)
    }

    fn build_request(
        &self,
        method: HttpMethod,
        url: &str,
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
        let mut req = self.client.request(req_method, url).headers(headers);
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
        let status_code = res.as_ref().ok().map(|r| r.status());
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
                        if let Err(e) = chunk {
                            return WorkerResult::StatsOnly {
                                success: false,
                                duration: start.elapsed(),
                                iteration,
                                status_code,
                                error: Some(e.to_string()),
                            };
                        }
                    }
                    WorkerResult::StatsOnly {
                        success: true,
                        duration: start.elapsed(),
                        iteration,
                        status_code,
                        error: None,
                    }
                }
                Err(e) => WorkerResult::StatsOnly {
                    success: false,
                    duration,
                    iteration,
                    status_code,
                    error: Some(e.to_string()),
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
        F: Fn(LoadTestEvent),
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
            let mut req_result = RequestResult {
                iteration: 0,
                duration: Duration::default(),
                success: false,
                status_code: None,
                error: None,
            };
            match worker_res {
                WorkerResult::StatsOnly {
                    success,
                    duration,
                    iteration,
                    status_code,
                    error,
                } => {
                    req_result.iteration = iteration;
                    req_result.duration = duration;
                    req_result.success = success;
                    req_result.status_code = status_code;
                    req_result.error = error;

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
                } => {
                    req_result.iteration = iteration;
                    req_result.duration = duration;
                    match res {
                        Ok(response) => {
                            req_result.success = true;
                            req_result.status_code = Some(response.status);
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
                            req_result.success = false;
                            req_result.error = Some(error.to_string());
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
                                    let _ =
                                        Self::write_failure_output_file(output_file, error).await;
                                });
                            }
                        }
                    }
                }
            }

            if !self.quiet {
                in_progress(LoadTestEvent::RequestFinished(req_result));

                if last_update.elapsed() >= update_interval {
                    result.elapsed = test_time.elapsed();
                    let elapsed_secs = result.elapsed.as_secs_f64();
                    if elapsed_secs > 0.0 {
                        result.rps = result.completed as f64 / elapsed_secs;
                    }
                    if result.completed > 0 {
                        result.success_rate =
                            (result.success as f64 / result.completed as f64) * 100.0;
                        result.failure_rate =
                            (result.failures as f64 / result.completed as f64) * 100.0;
                    }
                    result.update_percentiles();
                    in_progress(LoadTestEvent::ProgressUpdate(&result));
                    last_update = Instant::now();
                }
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
        in_progress(LoadTestEvent::ProgressUpdate(&result));
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
        let result = LoadTestRunner::builder("http://localhost:8080", 10, 2)
            .stats(Stats::Success)
            .build()
            .await
            .unwrap();

        assert_eq!(result.url, "http://localhost:8080");
        assert_eq!(result.requests, 10);
        assert_eq!(result.concurrency, 2);
    }

    #[tokio::test]
    async fn new_success() {
        let runner = LoadTestRunner::builder("http://localhost:8080", 10, 2)
            .stats(Stats::Success)
            .timeout(30)
            .user_agent("test-agent")
            .build()
            .await
            .unwrap();

        assert_eq!(runner.url, "http://localhost:8080");
        assert_eq!(runner.requests, 10);
        assert_eq!(runner.concurrency, 2);
        assert_eq!(runner.stats, Stats::Success);
    }

    #[tokio::test]
    async fn new_url_is_empty_fails() {
        let result = LoadTestRunner::builder("", 2, 2)
            .stats(Stats::Success)
            .build()
            .await
            .unwrap_err();

        assert_eq!(result.to_string(), "URL cannot be empty");
    }

    #[tokio::test]
    async fn new_num_requests_is_zero_fails() {
        let result = LoadTestRunner::builder("http://localhost:8080", 0, 2)
            .stats(Stats::Success)
            .build()
            .await
            .unwrap_err();

        assert_eq!(result.to_string(), "Number of requests cannot be zero");
    }

    #[tokio::test]
    async fn new_num_concurrency_is_zero_fails() {
        let result = LoadTestRunner::builder("http://localhost:8080", 2, 0)
            .stats(Stats::Success)
            .build()
            .await
            .unwrap_err();

        assert_eq!(result.to_string(), "Number of concurrency cannot be zero");
    }

    #[tokio::test]
    async fn new_num_concurrency_greater_than_num_requests_fails() {
        let result = LoadTestRunner::builder("http://localhost:8080", 2, 3)
            .stats(Stats::Success)
            .build()
            .await
            .unwrap_err();

        assert_eq!(
            result.to_string(),
            "Number of concurrency: 3 must be less than or equal to number of requests: 2"
        );
    }

    #[tokio::test]
    async fn new_ca_cert_does_not_exist_fails() {
        let result = LoadTestRunner::builder("http://localhost:8080", 10, 2)
            .stats(Stats::Success)
            .ca_cert("doesnotexist")
            .build()
            .await
            .unwrap_err();

        assert_eq!(
            result.to_string(),
            "CA certificate 'doesnotexist' does not exist or is not a file"
        );
    }

    #[tokio::test]
    async fn new_cert_does_not_exist_fails() {
        let result = LoadTestRunner::builder("http://localhost:8080", 10, 2)
            .stats(Stats::Success)
            .cert("doesnotexist")
            .key("tests/tls/key.pem")
            .build()
            .await
            .unwrap_err();

        assert_eq!(
            result.to_string(),
            "Certificate 'doesnotexist' does not exist or is not a file"
        );
    }

    #[tokio::test]
    async fn new_key_does_not_exist_fails() {
        let result = LoadTestRunner::builder("http://localhost:8080", 10, 2)
            .stats(Stats::Success)
            .cert("tests/tls/client.crt")
            .key("doesnotexist")
            .build()
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

        let content = String::from_utf8(bytes.to_vec())
            .unwrap()
            .replace("\r\n", "\n");
        assert_eq!(content, "{\n  \"message\": \"hello1\"\n}\n");
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
