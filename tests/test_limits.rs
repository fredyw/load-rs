use load_rs::{HttpMethod, LoadTestRunner, Stats};
use std::time::Duration;

mod common;

#[tokio::test]
async fn test_run_with_duration_limit() {
    let server = common::run_perf_server().await.unwrap();
    let url = format!("http://{}", server.addr);

    // Set a high number of requests but a short duration
    let runner = LoadTestRunner::builder(&url, 1000, 10)
        .duration(Duration::from_millis(200))
        .stats(Stats::All)
        .build()
        .await
        .unwrap();

    let result = runner
        .run(HttpMethod::Get, None, None, None, |_| {})
        .await
        .unwrap();

    // It should have stopped before 1000 requests
    assert!(result.completed < 1000);
    assert!(result.elapsed >= Duration::from_millis(200));
}

#[tokio::test]
async fn test_run_with_rps_limit() {
    let server = common::run_perf_server().await.unwrap();
    let url = format!("http://{}", server.addr);

    // Set RPS to 10. 20 requests should take about 2 seconds.
    let runner = LoadTestRunner::builder(&url, 20, 5)
        .rps(10)
        .stats(Stats::All)
        .build()
        .await
        .unwrap();

    let result = runner
        .run(HttpMethod::Get, None, None, None, |_| {})
        .await
        .unwrap();

    assert_eq!(result.completed, 20);
    // 20 requests at 10 RPS should take at least 1.9 seconds (roughly)
    assert!(result.elapsed >= Duration::from_millis(1500));
}
