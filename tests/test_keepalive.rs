use load_rs::{HttpMethod, LoadTestRunner, Stats};
use std::sync::atomic::Ordering;

mod common;
use common::*;

#[tokio::test]
async fn test_keepalive_enabled() {
    let test_server = run_perf_server().await.unwrap();

    let runner = LoadTestRunner::builder(format!("http://{}", test_server.addr), 10, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let result = runner
        .run(HttpMethod::Get, None, None, None, |_| {})
        .await
        .unwrap();

    assert_eq!(result.success, 10);
    // With keep-alive enabled, we should have FEWER connections than requests (ideally equal to concurrency).
    // In practice, it might be more due to how reqwest manages the pool, but it should be < 10.
    let total_conns = test_server.total_connections.load(Ordering::SeqCst);
    assert!(
        total_conns < 10,
        "Expected fewer than 10 connections with keep-alive, got {}",
        total_conns
    );
}

#[tokio::test]
async fn test_keepalive_disabled() {
    let test_server = run_perf_server().await.unwrap();

    let runner = LoadTestRunner::builder(format!("http://{}", test_server.addr), 10, 2)
        .stats(Stats::Success)
        .disable_keepalive(true)
        .build()
        .await
        .unwrap();

    let result = runner
        .run(HttpMethod::Get, None, None, None, |_| {})
        .await
        .unwrap();

    assert_eq!(result.success, 10);
    // With keep-alive disabled, we MUST have exactly one connection per request.
    let total_conns = test_server.total_connections.load(Ordering::SeqCst);
    assert_eq!(
        total_conns, 10,
        "Expected exactly 10 connections with keep-alive disabled, got {}",
        total_conns
    );
}
