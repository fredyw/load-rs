#![allow(dead_code)]

use anyhow::{Context, Result};
use http_body_util::Full;
use hyper::Response;
use hyper::body::Bytes;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig};
use rustls_pemfile::{certs, private_key};
use std::convert::Infallible;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::net::TcpListener;
use tokio::sync::{OnceCell, oneshot};
use tokio_rustls::TlsAcceptor;

use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::sleep;

static CRYPTO_PROVIDER: OnceCell<()> = OnceCell::const_new();

pub async fn install_crypto_provider() {
    CRYPTO_PROVIDER
        .get_or_init(|| async {
            rustls::crypto::aws_lc_rs::default_provider()
                .install_default()
                .unwrap();
        })
        .await;
}

pub struct TestServer {
    pub addr: SocketAddr,
    pub shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HttpVersion {
    Http1,
    Http2,
}

pub async fn run_server(version: HttpVersion) -> Result<TestServer> {
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

pub async fn load_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>> {
    let cert = fs::read(path).await?;
    let mut reader = BufReader::new(cert.as_slice());
    certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .context(format!("Failed to load certificate: {path:?}"))
}

pub async fn load_key(path: &Path) -> Result<PrivateKeyDer<'static>> {
    let key = fs::read(path).await?;
    let mut reader = std::io::BufReader::new(key.as_slice());
    private_key(&mut reader)?.context(format!("Failed to load key: {path:?}"))
}

pub struct PerformanceTestServer {
    pub addr: SocketAddr,
    pub max_active_connections: Arc<AtomicU32>,
    pub total_connections: Arc<AtomicU32>,
    pub shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Drop for PerformanceTestServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

pub async fn run_perf_server() -> Result<PerformanceTestServer> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = TcpListener::bind(addr).await?;
    let server_addr = listener.local_addr()?;
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
    let active_connections = Arc::new(AtomicU32::new(0));
    let max_active_connections = Arc::new(AtomicU32::new(0));
    let total_connections = Arc::new(AtomicU32::new(0));
    let active = Arc::clone(&active_connections);
    let max_active = Arc::clone(&max_active_connections);
    let total = Arc::clone(&total_connections);
    tokio::spawn(async move {
        loop {
            tokio::select! {
                res = listener.accept() => {
                    let (stream, _peer_addr) = match res {
                        Ok(res) => res,
                        Err(_) => continue,
                    };
                    total.fetch_add(1, Ordering::SeqCst);
                    let active = Arc::clone(&active);
                    let max_active = Arc::clone(&max_active);
                    tokio::spawn(async move {
                        let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                        loop {
                            let prev_max = max_active.load(Ordering::SeqCst);
                            if current <= prev_max {
                                break;
                            }
                            if max_active.compare_exchange(prev_max, current, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                                break;
                            }
                        }
                        let io = TokioIo::new(stream);
                        let service = service_fn(|_req| async {
                            // Delay to ensure concurrency can be observed.
                            sleep(Duration::from_millis(50)).await;
                            Ok::<_, Infallible>(Response::new(Full::new(Bytes::from("Hello"))))
                        });
                        if let Err(err) = hyper::server::conn::http1::Builder::new()
                            .serve_connection(io, service)
                            .await
                        {
                            eprintln!("Error serving connection: {:?}", err);
                        }
                        active.fetch_sub(1, Ordering::SeqCst);
                    });
                }
                _ = &mut shutdown_rx => {
                    break;
                }
            }
        }
    });
    Ok(PerformanceTestServer {
        addr: server_addr,
        max_active_connections,
        total_connections,
        shutdown_tx: Some(shutdown_tx),
    })
}

pub fn assert_result(
    result: &load_rs::LoadTestResult,
    expected_completed: u32,
    expected_success: u32,
) {
    assert_eq!(result.completed, expected_completed);
    assert_eq!(result.success, expected_success);
    assert_eq!(result.failures, expected_completed - expected_success);
    if expected_success > 0 {
        assert!(result.p50 >= Default::default());
        assert!(result.p90 >= Default::default());
        assert!(result.p95 >= Default::default());
        assert!(result.avg >= Default::default());
    }
}
