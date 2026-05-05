use load_rs::{HttpMethod, LoadTestRunner, Stats};
use std::sync::Arc;

use std::sync::atomic::{AtomicU32, Ordering};

mod common;
use common::*;

#[tokio::test]
async fn test_concurrency_limit() {
    let server = run_perf_server().await.unwrap();
    let url = format!("http://{}", server.addr);
    let concurrency = 3;
    let requests = 10;
    let runner = LoadTestRunner::new(
        &url,
        requests,
        concurrency,
        Stats::All,
        None,
        None,
        None,
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

    assert_eq!(result.completed, requests);
    assert_eq!(result.success, requests);

    let max_conn = server.max_active_connections.load(Ordering::SeqCst);
    // It should be exactly 'concurrency' because we have a delay and enough requests.
    assert!(
        max_conn <= concurrency,
        "Observed concurrency {} exceeded limit {}",
        max_conn,
        concurrency
    );
    assert!(max_conn > 0);
}

#[tokio::test]
async fn test_ui_debouncing() {
    let server = run_perf_server().await.unwrap();
    let url = format!("http://{}", server.addr);
    let requests = 20;
    let concurrency = 5;
    let runner = LoadTestRunner::new(
        &url,
        requests,
        concurrency,
        Stats::All,
        None,
        None,
        None,
        false,
        None,
        None,
    )
    .await
    .unwrap();
    let callback_count = Arc::new(AtomicU32::new(0));
    let cb = Arc::clone(&callback_count);
    let result = runner
        .run(HttpMethod::Get, None, None, None, move |event| {
            if let load_rs::LoadTestEvent::ProgressUpdate(_) = event {
                cb.fetch_add(1, Ordering::SeqCst);
            }
        })
        .await
        .unwrap();

    assert_eq!(result.completed, requests);

    let count = callback_count.load(Ordering::SeqCst);
    // With 20 requests and 50ms delay each, total time is roughly (20/5)*50ms = 200ms.
    // With 100ms debouncing, we expect roughly 2-4 calls.
    // Definitely less than 20.
    assert!(
        count < requests,
        "Callback count {} should be less than total requests {}",
        count,
        requests
    );
    assert!(
        count >= 1,
        "Callback should be called at least once at the end"
    );
}

#[tokio::test]
async fn run_with_timeout() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/delay/2",
        1,
        1,
        Stats::All,
        None,
        None,
        None,
        false,
        Some(1),
        None,
    )
    .await
    .unwrap();

    let result = runner
        .run(HttpMethod::Get, None, None, None, |_| {})
        .await
        .unwrap();

    assert_eq!(result.completed, 1);
    assert_eq!(result.success, 0);
    assert_eq!(result.failures, 1);
}

#[tokio::test]
async fn test_user_agent() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/get",
        1,
        1,
        Stats::All,
        None,
        None,
        None,
        false,
        None,
        Some("custom-agent/1.0"),
    )
    .await
    .unwrap();

    let resp = runner.debug(HttpMethod::Get, None, None).await.unwrap();

    let body = resp.text().await.unwrap();
    assert!(body.contains("\"user-agent\":\"custom-agent/1.0\""));
}

#[tokio::test]
async fn test_run_with_generator() {
    let server = run_perf_server().await.unwrap();
    let url = format!("http://{}", server.addr);

    let runner = LoadTestRunner::new(&url, 5, 2, Stats::All, None, None, None, false, None, None)
        .await
        .unwrap();

    let result = runner
        .run_with_generator(
            move |iteration: u64| {
                let req = reqwest::Client::new()
                    .get(&url)
                    .header("X-Iteration", iteration.to_string())
                    .build()?;
                Ok((req, None))
            },
            None,
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.completed, 5);
    assert_eq!(result.success, 5);
    assert_eq!(result.failures, 0);
}
