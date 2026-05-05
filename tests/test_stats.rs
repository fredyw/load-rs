use load_rs::Body::Data;
use load_rs::{HttpMethod, LoadTestRunner, Stats};
use reqwest::header::HeaderMap;

mod common;

#[tokio::test]
async fn run_success_stats() {
    // Successful requests with `Stats::Success`.
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        5,
        2,
        Stats::Success,
        None,
        None,
        None,
        false,
        None,
        None,
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
            None,
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

    // Failed requests with `Stats::Success`.
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        5,
        2,
        Stats::Success,
        None,
        None,
        None,
        false,
        None,
        None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run(
            HttpMethod::Post,
            Some(headers),
            Some(Data("hello".into())),
            None,
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 0);
    assert_eq!(result.failures, 5);
    assert_eq!(result.completed, 5);
    assert_eq!(result.p50, Default::default());
    assert_eq!(result.p90, Default::default());
    assert_eq!(result.p95, Default::default());
    assert_eq!(result.avg, Default::default());
}

#[tokio::test]
async fn run_failure_stats() {
    // Successful requests with `Stats::Error`.
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        5,
        2,
        Stats::Error,
        None,
        None,
        None,
        false,
        None,
        None,
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
            None,
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 5);
    assert_eq!(result.failures, 0);
    assert_eq!(result.completed, 5);
    assert_eq!(result.p50, Default::default());
    assert_eq!(result.p90, Default::default());
    assert_eq!(result.p95, Default::default());
    assert_eq!(result.avg, Default::default());

    // Failed requests with `Stats::Error`.
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        5,
        2,
        Stats::Error,
        None,
        None,
        None,
        false,
        None,
        None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run(
            HttpMethod::Post,
            Some(headers),
            Some(Data("hello".into())),
            None,
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 0);
    assert_eq!(result.failures, 5);
    assert_eq!(result.completed, 5);
    assert!(result.p50 > Default::default());
    assert!(result.p90 > Default::default());
    assert!(result.p95 > Default::default());
    assert!(result.avg > Default::default());
}

#[tokio::test]
async fn run_all_stats() {
    // Successful requests with `Stats::All`.
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        5,
        2,
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

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run(
            HttpMethod::Post,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
            None,
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

    // Failed requests with `Stats::Success`.
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        5,
        2,
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

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run(
            HttpMethod::Post,
            Some(headers),
            Some(Data("hello".into())),
            None,
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 0);
    assert_eq!(result.failures, 5);
    assert_eq!(result.completed, 5);
    assert!(result.p50 > Default::default());
    assert!(result.p90 > Default::default());
    assert!(result.p95 > Default::default());
    assert!(result.avg > Default::default());
}
