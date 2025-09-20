extern crate load_rs;

use load_rs::Body::Data;
use load_rs::{HttpMethod, LoadTestRunner, Order};
use reqwest::header::HeaderMap;

#[tokio::test]
async fn run_get() {
    let runner = LoadTestRunner::new("https://mockhttp.org/get", 5, 2, &None, &None, &None, &None)
        .await
        .unwrap();

    let result = runner
        .run(HttpMethod::Get, None, None, |_| {})
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
    let runner = LoadTestRunner::new("https://mockhttp.org/get", 5, 2, &None, &None, &None, &None)
        .await
        .unwrap();

    let result = runner
        .run(HttpMethod::Head, None, None, |_| {})
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
        &None,
        &None,
        &None,
        &None,
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
    let runner = LoadTestRunner::new("https://mockhttp.org/put", 5, 2, &None, &None, &None, &None)
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run(
            HttpMethod::Put,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
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
        &None,
        &None,
        &None,
        &None,
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
        &None,
        &None,
        &None,
        &None,
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
async fn run_from_dir_post_randoml() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        5,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Post,
            Some(headers),
            &"tests/test_requests".into(),
            Order::Random,
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
async fn run_from_dir_put_random() {
    let runner = LoadTestRunner::new("https://mockhttp.org/put", 5, 2, &None, &None, &None, &None)
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Put,
            Some(headers),
            &"tests/test_requests".into(),
            Order::Random,
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
async fn run_from_dir_patch_random() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/patch",
        5,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Patch,
            Some(headers),
            &"tests/test_requests".into(),
            Order::Random,
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
async fn run_from_dir_delete_random() {
    let runner = LoadTestRunner::new(
        "https://mockhttp.org/delete",
        5,
        2,
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Delete,
            Some(headers),
            &"tests/test_requests".into(),
            Order::Random,
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
