use load_rs::{HttpMethod, LoadTestRunner, Stats};

mod common;
use common::*;

#[tokio::test]
async fn run_mtls_http1_valid_certs() {
    let test_server = run_server(HttpVersion::Http1).await.unwrap();

    let runner = LoadTestRunner::builder(format!("https://{}", test_server.addr), 5, 2)
        .stats(Stats::Success)
        .ca_cert("tests/tls/ca.crt")
        .cert("tests/tls/client.crt")
        .key("tests/tls/client.key")
        .build()
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

    let runner = LoadTestRunner::builder(format!("https://{}", test_server.addr), 5, 2)
        .stats(Stats::Success)
        .ca_cert("tests/tls/ca.crt")
        .cert("tests/tls/client.crt")
        .key("tests/tls/client.key")
        .build()
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

    let runner = LoadTestRunner::builder(format!("https://{}", test_server.addr), 5, 2)
        .stats(Stats::Success)
        .ca_cert("tests/tls/untrusted-ca.crt")
        .cert("tests/tls/untrusted-client.crt")
        .key("tests/tls/untrusted-client.key")
        .build()
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
