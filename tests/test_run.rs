use load_rs::Body::{Data, DataFile};
use load_rs::{HttpMethod, LoadTestRunner, Order, Stats};
use reqwest::header::HeaderMap;
use std::path::Path;

mod common;
use common::*;

#[tokio::test]
async fn run_get() {
    let runner = LoadTestRunner::builder("https://mockhttp.org/get", 5, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let result = runner
        .run(HttpMethod::Get, None, None, None, |_| {})
        .await
        .unwrap();

    assert_result(&result, 5, 5);
}

#[tokio::test]
async fn run_head() {
    let runner = LoadTestRunner::builder("https://mockhttp.org/get", 5, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let result = runner
        .run(HttpMethod::Head, None, None, None, |_| {})
        .await
        .unwrap();

    assert_result(&result, 5, 5);
}

#[tokio::test]
async fn run_post() {
    let runner = LoadTestRunner::builder("https://mockhttp.org/post", 5, 2)
        .stats(Stats::Success)
        .build()
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

    assert_result(&result, 5, 5);
}

#[tokio::test]
async fn run_put() {
    let runner = LoadTestRunner::builder("https://mockhttp.org/put", 5, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run(
            HttpMethod::Put,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
            None,
            |_| {},
        )
        .await
        .unwrap();

    assert_result(&result, 5, 5);
}

#[tokio::test]
async fn run_patch() {
    let runner = LoadTestRunner::builder("https://mockhttp.org/patch", 5, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run(
            HttpMethod::Patch,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
            None,
            |_| {},
        )
        .await
        .unwrap();

    assert_result(&result, 5, 5);
}

#[tokio::test]
async fn run_delete() {
    let runner = LoadTestRunner::builder("https://mockhttp.org/delete", 5, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run(
            HttpMethod::Delete,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
            None,
            |_| {},
        )
        .await
        .unwrap();

    assert_result(&result, 5, 5);
}

#[tokio::test]
async fn run_from_dir_sequential() {
    let runner = LoadTestRunner::builder("https://mockhttp.org/post", 5, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Post,
            Some(headers),
            Path::new("tests/test_requests"),
            Order::Sequential,
            None,
            |_| {},
        )
        .await
        .unwrap();

    assert_result(&result, 5, 5);
}

#[tokio::test]
async fn run_from_data_file() {
    let runner = LoadTestRunner::builder("https://mockhttp.org/post", 5, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run(
            HttpMethod::Post,
            Some(headers),
            Some(DataFile("tests/test_requests/test1.json".into())),
            None,
            |_| {},
        )
        .await
        .unwrap();

    assert_result(&result, 5, 5);
}

#[tokio::test]
async fn run_from_dir_random() {
    let runner = LoadTestRunner::builder("https://mockhttp.org/post", 5, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Post,
            Some(headers),
            Path::new("tests/test_requests"),
            Order::Random,
            None,
            |_| {},
        )
        .await
        .unwrap();

    assert_result(&result, 5, 5);
}

#[tokio::test]
async fn run_from_dir_requests_less_than_files_sequential() {
    let runner = LoadTestRunner::builder("https://mockhttp.org/post", 3, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Post,
            Some(headers),
            Path::new("tests/test_requests"),
            Order::Sequential,
            None,
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
    let runner = LoadTestRunner::builder("https://mockhttp.org/post", 7, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Post,
            Some(headers),
            Path::new("tests/test_requests"),
            Order::Sequential,
            None,
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
    let runner = LoadTestRunner::builder("https://mockhttp.org/post", 3, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Post,
            Some(headers),
            Path::new("tests/test_requests"),
            Order::Random,
            None,
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
    let runner = LoadTestRunner::builder("https://mockhttp.org/post", 7, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Post,
            Some(headers),
            Path::new("tests/test_requests"),
            Order::Random,
            None,
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
async fn run_from_manifest_random() {
    let runner = LoadTestRunner::builder("https://mockhttp.org/post", 5, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let result = runner
        .run_from_manifest(
            HttpMethod::Post,
            Path::new("tests/test_manifests/requests.jsonl"),
            Order::Random,
            None,
            |_| {},
        )
        .await
        .unwrap();

    assert_result(&result, 5, 5);
}

#[tokio::test]
async fn run_from_manifest_requests_less_than_files_sequential() {
    let runner = LoadTestRunner::builder("https://mockhttp.org/post", 3, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let result = runner
        .run_from_manifest(
            HttpMethod::Post,
            Path::new("tests/test_manifests/requests.jsonl"),
            Order::Sequential,
            None,
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
async fn run_from_manifest_requests_greater_than_files_sequential() {
    let runner = LoadTestRunner::builder("https://mockhttp.org/post", 7, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let result = runner
        .run_from_manifest(
            HttpMethod::Post,
            Path::new("tests/test_manifests/requests.jsonl"),
            Order::Sequential,
            None,
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
async fn run_from_manifest_requests_less_than_files_random() {
    let runner = LoadTestRunner::builder("https://mockhttp.org/post", 3, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let result = runner
        .run_from_manifest(
            HttpMethod::Post,
            Path::new("tests/test_manifests/requests.jsonl"),
            Order::Random,
            None,
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
async fn run_from_manifest_requests_greater_than_files_random() {
    let runner = LoadTestRunner::builder("https://mockhttp.org/post", 7, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let result = runner
        .run_from_manifest(
            HttpMethod::Post,
            Path::new("tests/test_manifests/requests.jsonl"),
            Order::Random,
            None,
            |_| {},
        )
        .await
        .unwrap();

    assert_result(&result, 7, 7);
}

#[tokio::test]
async fn run_with_timeout() {
    let runner = LoadTestRunner::builder("https://mockhttp.org/delay/2", 1, 1)
        .stats(Stats::All)
        .timeout(1)
        .build()
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
async fn test_run_with_generator() {
    let server = run_perf_server().await.unwrap();
    let url = format!("http://{}", server.addr);

    let runner = LoadTestRunner::builder(&url, 5, 2)
        .stats(Stats::All)
        .build()
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

    assert_result(&result, 5, 5);
}

#[tokio::test]
async fn run_from_empty_dir_fails() {
    let runner = LoadTestRunner::builder("http://localhost:8080", 5, 2)
        .stats(Stats::All)
        .build()
        .await
        .unwrap();

    let result = runner
        .run_from_dir(
            HttpMethod::Post,
            None,
            std::path::Path::new("tests/empty_dir"),
            Order::Sequential,
            None,
            |_| {},
        )
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn run_from_manifest_with_overrides() {
    let runner = LoadTestRunner::builder("https://mockhttp.org", 3, 1)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let result = runner
        .run_from_manifest(
            HttpMethod::Get,
            Path::new("tests/test_manifests/overrides.jsonl"),
            Order::Sequential,
            None,
            |_| {},
        )
        .await
        .unwrap();

    assert_result(&result, 3, 3);
}
