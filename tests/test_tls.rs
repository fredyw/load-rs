use load_rs::{HttpMethod, LoadTestRunner, Stats};
use std::path::Path;

mod common;
use common::*;

#[tokio::test]
async fn run_mtls_http1_valid_certs() {
    let test_server = run_server(HttpVersion::Http1).await.unwrap();

    let runner = LoadTestRunner::new(
        format!("https://{}", test_server.addr).as_str(),
        5,
        2,
        Stats::Success,
        Some(Path::new("tests/tls/ca.crt")),
        Some(Path::new("tests/tls/client.crt")),
        Some(Path::new("tests/tls/client.key")),
        false,
        None,
        None,
    )
    .await
    .unwrap();

    let result = runner
        .run(HttpMethod::Get, None, None, None, |_| {})
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
        Stats::Success,
        Some(Path::new("tests/tls/ca.crt")),
        Some(Path::new("tests/tls/client.crt")),
        Some(Path::new("tests/tls/client.key")),
        false,
        None,
        None,
    )
    .await
    .unwrap();

    let result = runner
        .run(HttpMethod::Get, None, None, None, |_| {})
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
        Stats::Success,
        Some(Path::new("tests/tls/untrusted-ca.crt")),
        Some(Path::new("tests/tls/untrusted-client.crt")),
        Some(Path::new("tests/tls/untrusted-client.key")),
        false,
        None,
        None,
    )
    .await
    .unwrap();

    let result = runner
        .run(HttpMethod::Get, None, None, None, |_| {})
        .await
        .unwrap();

    assert_eq!(result.success, 0);
    assert_eq!(result.failures, 5);
    assert_eq!(result.completed, 5);
}
