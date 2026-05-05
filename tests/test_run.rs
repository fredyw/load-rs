use load_rs::Body::{Data, DataFile};
use load_rs::{HttpMethod, LoadTestRunner, Order, Stats};
use reqwest::header::HeaderMap;
use std::path::Path;

mod common;

#[tokio::test]
async fn run_get() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/get",
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
async fn run_head() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/get",
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

    let result = runner
        .run(HttpMethod::Head, None, None, None, |_| {})
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
}

#[tokio::test]
async fn run_put() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/put",
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
            HttpMethod::Put,
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
}

#[tokio::test]
async fn run_patch() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/patch",
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
            HttpMethod::Patch,
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
}

#[tokio::test]
async fn run_delete() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/delete",
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
            HttpMethod::Delete,
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
}

#[tokio::test]
async fn run_from_dir_sequential() {
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
            Some(DataFile("tests/test_requests/test1.json".into())),
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
}

#[tokio::test]
async fn run_from_dir_random() {
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
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        7,
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
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        3,
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
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        7,
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

    let result = runner
        .run_from_manifest(
            HttpMethod::Post,
            Path::new("tests/test_manifests/manifest1.jsonl"),
            Order::Random,
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
}

#[tokio::test]
async fn run_from_manifest_requests_less_than_files_sequential() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        3,
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

    let result = runner
        .run_from_manifest(
            HttpMethod::Post,
            Path::new("tests/test_manifests/manifest1.jsonl"),
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
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        7,
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

    let result = runner
        .run_from_manifest(
            HttpMethod::Post,
            Path::new("tests/test_manifests/manifest1.jsonl"),
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
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        3,
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

    let result = runner
        .run_from_manifest(
            HttpMethod::Post,
            Path::new("tests/test_manifests/manifest1.jsonl"),
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
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        7,
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

    let result = runner
        .run_from_manifest(
            HttpMethod::Post,
            Path::new("tests/test_manifests/manifest1.jsonl"),
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
