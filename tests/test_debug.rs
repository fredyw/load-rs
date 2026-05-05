use load_rs::Body::{Data, DataFile};
use load_rs::{HttpMethod, LoadTestRunner, Order, Stats};
use reqwest::header::HeaderMap;
use std::path::Path;

mod common;

#[tokio::test]
async fn debug_get() {
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
        None,
    )
    .await
    .unwrap();

    let response = runner.debug(HttpMethod::Get, None, None).await.unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_head() {
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
        None,
    )
    .await
    .unwrap();

    let response = runner.debug(HttpMethod::Head, None, None).await.unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_post() {
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
        None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let response = runner
        .debug(
            HttpMethod::Post,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_put() {
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
        None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let response = runner
        .debug(
            HttpMethod::Put,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_patch() {
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
        None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let response = runner
        .debug(
            HttpMethod::Patch,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_delete() {
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
        None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let response = runner
        .debug(
            HttpMethod::Delete,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_from_data_file() {
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
        None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let response = runner
        .debug(
            HttpMethod::Post,
            Some(headers),
            Some(DataFile("tests/test_requests/test1.json".into())),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_from_dir_sequential() {
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
        None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let response = runner
        .debug_from_dir(
            HttpMethod::Post,
            Some(headers),
            Path::new("tests/test_requests"),
            Order::Sequential,
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_from_dir_random() {
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
        None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let response = runner
        .debug_from_dir(
            HttpMethod::Post,
            Some(headers),
            Path::new("tests/test_requests"),
            Order::Random,
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_from_manifest_sequential() {
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
        None,
    )
    .await
    .unwrap();

    let response = runner
        .debug_from_manifest(
            HttpMethod::Post,
            Path::new("tests/test_manifests/manifest1.jsonl"),
            Order::Sequential,
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_from_manifest_random() {
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
        None,
    )
    .await
    .unwrap();

    let response = runner
        .debug_from_manifest(
            HttpMethod::Post,
            Path::new("tests/test_manifests/manifest1.jsonl"),
            Order::Random,
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
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
        None,
    )
    .await
    .unwrap();

    let resp = runner.debug(HttpMethod::Get, None, None).await.unwrap();

    let body = resp.text().await.unwrap();
    assert!(body.contains("\"user-agent\":\"custom-agent/1.0\""));
}
