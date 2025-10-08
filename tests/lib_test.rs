extern crate load_rs;

use anyhow::{Context, Result};
use http_body_util::Full;
use hyper::Response;
use hyper::body::Bytes;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use load_rs::Body::{Data, DataFile};
use load_rs::{HttpMethod, LoadTestRunner, Order};
use reqwest::header::HeaderMap;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig};
use rustls_pemfile::{certs, private_key};
use std::convert::Infallible;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::net::TcpListener;
use tokio::sync::{OnceCell, oneshot};
use tokio_rustls::TlsAcceptor;

static CRYPTO_PROVIDER: OnceCell<()> = OnceCell::const_new();

async fn install_crypto_provider() {
    CRYPTO_PROVIDER
        .get_or_init(|| async {
            rustls::crypto::aws_lc_rs::default_provider()
                .install_default()
                .unwrap();
        })
        .await;
}

struct TestServer {
    addr: SocketAddr,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum HttpVersion {
    Http1,
    Http2,
}

async fn run_server(version: HttpVersion) -> Result<TestServer> {
    install_crypto_provider().await;
    let provider = rustls::crypto::aws_lc_rs::default_provider();

    let ca_cert = Path::new("tests/tls/ca.crt");
    let server_cert = Path::new("tests/tls/server.crt");
    let server_key = Path::new("tests/tls/server.key");

    let mut root_store = RootCertStore::empty();
    let ca_certs = load_certs(ca_cert).await?;
    root_store.add_parsable_certificates(ca_certs);
    let client_auth = WebPkiClientVerifier::builder(root_store.into()).build()?;

    let mut server_config = ServerConfig::builder_with_provider(provider.into())
        .with_protocol_versions(&[&rustls::version::TLS12, &rustls::version::TLS13])?
        .with_client_cert_verifier(client_auth)
        .with_single_cert(load_certs(server_cert).await?, load_key(server_key).await?)
        .context("Failed to create TLS server config")?;
    if version == HttpVersion::Http2 {
        server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    }

    let acceptor = TlsAcceptor::from(Arc::new(server_config));
    let service = service_fn(|_req| async {
        Ok::<_, Infallible>(Response::new(Full::new(Bytes::from("Hello"))))
    });
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = TcpListener::bind(addr).await?;
    let server_addr = listener.local_addr()?;
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel();

    tokio::spawn(async move {
        loop {
            tokio::select! {
               res = listener.accept() => {
                    let (stream, _peer_addr) = match res {
                        Ok(res) => res,
                        Err(_) => continue,
                    };
                    let acceptor = acceptor.clone();
                    let service = service;
                    tokio::spawn(async move {
                        if let Ok(tls_stream) = acceptor.accept(stream).await {
                            let io = TokioIo::new(tls_stream);
                            match version {
                                HttpVersion::Http1 => {
                                    hyper::server::conn::http1::Builder::new().serve_connection(io, service).await.ok();
                                }
                                HttpVersion::Http2 => {
                                    hyper::server::conn::http2::Builder::new(TokioExecutor::new())
                                        .serve_connection(io, service)
                                        .await
                                        .ok();
                                }
                            }
                        }
                    });
                },
                _ = &mut shutdown_rx => {
                    break;
                }
            }
        }
    });

    Ok(TestServer {
        addr: server_addr,
        shutdown_tx: Some(shutdown_tx),
    })
}

async fn load_certs(path: &Path) -> Result<Vec<CertificateDer>> {
    let cert = fs::read(path).await?;
    let mut reader = BufReader::new(cert.as_slice());
    certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .context(format!("Failed to load certificate: {path:?}"))
}

async fn load_key(path: &Path) -> Result<PrivateKeyDer> {
    let key = fs::read(path).await?;
    let mut reader = std::io::BufReader::new(key.as_slice());
    private_key(&mut reader)?.context(format!("Failed to load key: {path:?}"))
}

#[tokio::test]
async fn run_get() {
    let runner = LoadTestRunner::new("https://mockhttp.org/get", 5, 2, &None, &None, &None, &None)
        .await
        .unwrap();

    let result = runner
        .run(HttpMethod::Get, None, None, &None, |_| {})
        .await
        .unwrap();

    assert_eq!(result.success, 5);
    assert_eq!(result.failures, 0);
    assert_eq!(result.completed, 5);
    assert!(result.p50 > Default::default());
    assert!(result.p90 > Default::default());
    assert!(result.p95 > Default::default());
    assert!(result.avg > Default::default());
}

#[tokio::test]
async fn run_head() {
    let runner = LoadTestRunner::new("https://mockhttp.org/get", 5, 2, &None, &None, &None, &None)
        .await
        .unwrap();

    let result = runner
        .run(HttpMethod::Head, None, None, &None, |_| {})
        .await
        .unwrap();

    assert_eq!(result.success, 5);
    assert_eq!(result.failures, 0);
    assert_eq!(result.completed, 5);
    assert!(result.p50 > Default::default());
    assert!(result.p90 > Default::default());
    assert!(result.p95 > Default::default());
    assert!(result.avg > Default::default());
}

#[tokio::test]
async fn run_post() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        5,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run(
            HttpMethod::Post,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
            &None,
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 5);
    assert_eq!(result.failures, 0);
    assert_eq!(result.completed, 5);
    assert!(result.p50 > Default::default());
    assert!(result.p90 > Default::default());
    assert!(result.p95 > Default::default());
    assert!(result.avg > Default::default());
}

#[tokio::test]
async fn run_put() {
    let runner = LoadTestRunner::new("https://mockhttp.org/put", 5, 2, &None, &None, &None, &None)
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run(
            HttpMethod::Put,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
            &None,
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 5);
    assert_eq!(result.failures, 0);
    assert_eq!(result.completed, 5);
    assert!(result.p50 > Default::default());
    assert!(result.p90 > Default::default());
    assert!(result.p95 > Default::default());
    assert!(result.avg > Default::default());
}

#[tokio::test]
async fn run_patch() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/patch",
        5,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run(
            HttpMethod::Patch,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
            &None,
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 5);
    assert_eq!(result.failures, 0);
    assert_eq!(result.completed, 5);
    assert!(result.p50 > Default::default());
    assert!(result.p90 > Default::default());
    assert!(result.p95 > Default::default());
    assert!(result.avg > Default::default());
}

#[tokio::test]
async fn run_delete() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/delete",
        5,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run(
            HttpMethod::Delete,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
            &None,
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 5);
    assert_eq!(result.failures, 0);
    assert_eq!(result.completed, 5);
    assert!(result.p50 > Default::default());
    assert!(result.p90 > Default::default());
    assert!(result.p95 > Default::default());
    assert!(result.avg > Default::default());
}

#[tokio::test]
async fn run_from_dir_sequential() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        5,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Post,
            Some(headers),
            &"tests/test_requests".into(),
            Order::Sequential,
            &None,
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 5);
    assert_eq!(result.failures, 0);
    assert_eq!(result.completed, 5);
    assert!(result.p50 > Default::default());
    assert!(result.p90 > Default::default());
    assert!(result.p95 > Default::default());
    assert!(result.avg > Default::default());
}

#[tokio::test]
async fn run_from_data_file() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        5,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run(
            HttpMethod::Post,
            Some(headers),
            Some(DataFile("tests/test_requests/test1.json".into())),
            &None,
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 5);
    assert_eq!(result.failures, 0);
    assert_eq!(result.completed, 5);
    assert!(result.p50 > Default::default());
    assert!(result.p90 > Default::default());
    assert!(result.p95 > Default::default());
    assert!(result.avg > Default::default());
}

#[tokio::test]
async fn run_from_dir_random() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        5,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Post,
            Some(headers),
            &"tests/test_requests".into(),
            Order::Random,
            &None,
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 5);
    assert_eq!(result.failures, 0);
    assert_eq!(result.completed, 5);
    assert!(result.p50 > Default::default());
    assert!(result.p90 > Default::default());
    assert!(result.p95 > Default::default());
    assert!(result.avg > Default::default());
}

#[tokio::test]
async fn run_from_dir_requests_less_than_files_sequential() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        3,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Post,
            Some(headers),
            &"tests/test_requests".into(),
            Order::Sequential,
            &None,
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 3);
    assert_eq!(result.failures, 0);
    assert_eq!(result.completed, 3);
    assert!(result.p50 > Default::default());
    assert!(result.p90 > Default::default());
    assert!(result.p95 > Default::default());
    assert!(result.avg > Default::default());
}

#[tokio::test]
async fn run_from_dir_requests_greater_than_files_sequential() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        7,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Post,
            Some(headers),
            &"tests/test_requests".into(),
            Order::Sequential,
            &None,
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 7);
    assert_eq!(result.failures, 0);
    assert_eq!(result.completed, 7);
    assert!(result.p50 > Default::default());
    assert!(result.p90 > Default::default());
    assert!(result.p95 > Default::default());
    assert!(result.avg > Default::default());
}

#[tokio::test]
async fn run_from_dir_requests_less_than_files_random() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        3,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Post,
            Some(headers),
            &"tests/test_requests".into(),
            Order::Random,
            &None,
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 3);
    assert_eq!(result.failures, 0);
    assert_eq!(result.completed, 3);
    assert!(result.p50 > Default::default());
    assert!(result.p90 > Default::default());
    assert!(result.p95 > Default::default());
    assert!(result.avg > Default::default());
}

#[tokio::test]
async fn run_from_dir_requests_greater_than_files_random() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        7,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Post,
            Some(headers),
            &"tests/test_requests".into(),
            Order::Random,
            &None,
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 7);
    assert_eq!(result.failures, 0);
    assert_eq!(result.completed, 7);
    assert!(result.p50 > Default::default());
    assert!(result.p90 > Default::default());
    assert!(result.p95 > Default::default());
    assert!(result.avg > Default::default());
}

#[tokio::test]
async fn debug_get() {
    let runner = LoadTestRunner::new("https://mockhttp.org/get", 5, 2, &None, &None, &None, &None)
        .await
        .unwrap();

    let response = runner.debug(HttpMethod::Get, None, None).await.unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_head() {
    let runner = LoadTestRunner::new("https://mockhttp.org/get", 5, 2, &None, &None, &None, &None)
        .await
        .unwrap();

    let response = runner.debug(HttpMethod::Head, None, None).await.unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_post() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        5,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let response = runner
        .debug(
            HttpMethod::Post,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_put() {
    let runner = LoadTestRunner::new("https://mockhttp.org/put", 5, 2, &None, &None, &None, &None)
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let response = runner
        .debug(
            HttpMethod::Put,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_patch() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/patch",
        5,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let response = runner
        .debug(
            HttpMethod::Patch,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_delete() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/delete",
        5,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let response = runner
        .debug(
            HttpMethod::Delete,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_from_data_file() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        5,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let response = runner
        .debug(
            HttpMethod::Post,
            Some(headers),
            Some(DataFile("tests/test_requests/test1.json".into())),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_from_dir_sequential() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        5,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let response = runner
        .debug_from_dir(
            HttpMethod::Post,
            Some(headers),
            &"tests/test_requests".into(),
            Order::Sequential,
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_from_dir_random() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        5,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let response = runner
        .debug_from_dir(
            HttpMethod::Post,
            Some(headers),
            &"tests/test_requests".into(),
            Order::Random,
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn run_success_save_responses() {
    let dir = "/tmp/load-rs/lib1";
    let output_dir: PathBuf = dir.into();
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).await.unwrap();
    }

    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        3,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run(
            HttpMethod::Post,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
            &Some(output_dir),
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 3);
    assert!(PathBuf::from(format!("{dir}/success-1.json")).exists());
    assert!(PathBuf::from(format!("{dir}/success-2.json")).exists());
    assert!(PathBuf::from(format!("{dir}/success-3.json")).exists());
}

#[tokio::test]
async fn run_failure_save_responses() {
    let dir = "/tmp/load-rs/lib2";
    let output_dir: PathBuf = dir.into();
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).await.unwrap();
    }

    let runner = LoadTestRunner::new("https://mockhttp.org/get", 3, 2, &None, &None, &None, &None)
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run(
            HttpMethod::Post,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
            &Some(output_dir),
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.failures, 3);
    assert!(PathBuf::from(format!("{dir}/failure-1.json")).exists());
    assert!(PathBuf::from(format!("{dir}/failure-2.json")).exists());
    assert!(PathBuf::from(format!("{dir}/failure-3.json")).exists());
}

#[tokio::test]
async fn run_from_dir_success_save_responses() {
    let dir = "/tmp/load-rs/lib3";
    let output_dir: PathBuf = dir.into();
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).await.unwrap();
    }

    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        3,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Post,
            Some(headers),
            &"tests/test_requests".into(),
            Order::Sequential,
            &Some(output_dir),
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 3);
    assert!(PathBuf::from(format!("{dir}/success-1-test1.json")).exists());
    assert!(PathBuf::from(format!("{dir}/success-2-test2.json")).exists());
    assert!(PathBuf::from(format!("{dir}/success-3-test3.json")).exists());
}

#[tokio::test]
async fn run_from_dir_failure_save_responses() {
    let dir = "/tmp/load-rs/lib4";
    let output_dir: PathBuf = dir.into();
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).await.unwrap();
    }

    let runner = LoadTestRunner::new("https://mockhttp.org/get", 3, 2, &None, &None, &None, &None)
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Post,
            Some(headers),
            &"tests/test_requests".into(),
            Order::Sequential,
            &Some(output_dir),
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.failures, 3);
    assert!(PathBuf::from(format!("{dir}/failure-1-test1.json")).exists());
    assert!(PathBuf::from(format!("{dir}/failure-2-test2.json")).exists());
    assert!(PathBuf::from(format!("{dir}/failure-3-test3.json")).exists());
}

#[tokio::test]
async fn run_mtls_http1_valid_certs() {
    let test_server = run_server(HttpVersion::Http1).await.unwrap();

    let runner = LoadTestRunner::new(
        format!("https://{}", test_server.addr).as_str(),
        5,
        2,
        &Some("tests/tls/ca.crt".into()),
        &Some("tests/tls/client.crt".into()),
        &Some("tests/tls/client.key".into()),
        &None,
    )
    .await
    .unwrap();

    let result = runner
        .run(HttpMethod::Get, None, None, &None, |_| {})
        .await
        .unwrap();

    assert_eq!(result.success, 5);
    assert_eq!(result.failures, 0);
    assert_eq!(result.completed, 5);
    assert!(result.p50 > Default::default());
    assert!(result.p90 > Default::default());
    assert!(result.p95 > Default::default());
    assert!(result.avg > Default::default());
}

#[tokio::test]
async fn run_mtls_http2_valid_certs() {
    let test_server = run_server(HttpVersion::Http2).await.unwrap();

    let runner = LoadTestRunner::new(
        format!("https://{}", test_server.addr).as_str(),
        5,
        2,
        &Some("tests/tls/ca.crt".into()),
        &Some("tests/tls/client.crt".into()),
        &Some("tests/tls/client.key".into()),
        &None,
    )
    .await
    .unwrap();

    let result = runner
        .run(HttpMethod::Get, None, None, &None, |_| {})
        .await
        .unwrap();

    assert_eq!(result.success, 5);
    assert_eq!(result.failures, 0);
    assert_eq!(result.completed, 5);
    assert!(result.p50 > Default::default());
    assert!(result.p90 > Default::default());
    assert!(result.p95 > Default::default());
    assert!(result.avg > Default::default());
}

#[tokio::test]
async fn run_mtls_invalid_certs() {
    let test_server = run_server(HttpVersion::Http2).await.unwrap();

    let runner = LoadTestRunner::new(
        format!("https://{}", test_server.addr).as_str(),
        5,
        2,
        &Some("tests/tls/untrusted-ca.crt".into()),
        &Some("tests/tls/untrusted-client.crt".into()),
        &Some("tests/tls/untrusted-client.key".into()),
        &None,
    )
    .await
    .unwrap();

    let result = runner
        .run(HttpMethod::Get, None, None, &None, |_| {})
        .await
        .unwrap();

    assert_eq!(result.success, 0);
    assert_eq!(result.failures, 5);
    assert_eq!(result.completed, 5);
}
