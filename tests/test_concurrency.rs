use load_rs::{HttpMethod, LoadTestRunner, Stats};
use std::sync::atomic::Ordering;

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
