use load_rs::Body::Data;
use load_rs::{HttpMethod, LoadTestRunner, Order, Stats};
use reqwest::header::HeaderMap;
use std::path::Path;
use tokio::fs;

mod common;

#[tokio::test]
async fn run_success_save_responses() {
    let output_file = std::env::temp_dir().join("load-rs-lib1.jsonl");
    if output_file.exists() {
        fs::remove_file(&output_file).await.unwrap();
    }

    let runner = LoadTestRunner::builder("https://mockhttp.org/post", 3, 2)
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
            Some(&output_file),
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 3);
    assert!(output_file.exists());
    let content = fs::read_to_string(&output_file).await.unwrap();
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(lines.len(), 3);
}

#[tokio::test]
async fn run_failure_save_responses() {
    let output_file = std::env::temp_dir().join("load-rs-lib2.jsonl");
    if output_file.exists() {
        fs::remove_file(&output_file).await.unwrap();
    }

    let runner = LoadTestRunner::builder("https://mockhttp.org/get", 3, 2)
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
            Some(&output_file),
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.failures, 3);
    assert!(output_file.exists());
    let content = fs::read_to_string(&output_file).await.unwrap();
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(lines.len(), 3);
}

#[tokio::test]
async fn run_from_dir_success_save_responses() {
    let output_file = std::env::temp_dir().join("load-rs-lib3.jsonl");
    if output_file.exists() {
        fs::remove_file(&output_file).await.unwrap();
    }

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
            Some(&output_file),
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 3);
    assert!(output_file.exists());
    let content = fs::read_to_string(&output_file).await.unwrap();
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(lines.len(), 3);
}

#[tokio::test]
async fn run_from_dir_failure_save_responses() {
    let output_file = std::env::temp_dir().join("load-rs-lib4.jsonl");
    if output_file.exists() {
        fs::remove_file(&output_file).await.unwrap();
    }

    let runner = LoadTestRunner::builder("https://mockhttp.org/get", 3, 2)
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
            Some(&output_file),
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.failures, 3);
    assert!(output_file.exists());
    let content = fs::read_to_string(&output_file).await.unwrap();
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(lines.len(), 3);
}

#[tokio::test]
async fn run_from_manifest_success_save_responses() {
    let output_file = std::env::temp_dir().join("load-rs-lib5.jsonl");
    if output_file.exists() {
        fs::remove_file(&output_file).await.unwrap();
    }

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
            Some(&output_file),
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 3);
    assert!(output_file.exists());
    let content = fs::read_to_string(&output_file).await.unwrap();
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(lines.len(), 3);
}

#[tokio::test]
async fn run_from_manifest_failure_save_responses() {
    let output_file = std::env::temp_dir().join("load-rs-lib6.jsonl");
    if output_file.exists() {
        fs::remove_file(&output_file).await.unwrap();
    }

    let runner = LoadTestRunner::builder("https://mockhttp.org/get", 3, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let result = runner
        .run_from_manifest(
            HttpMethod::Post,
            Path::new("tests/test_manifests/requests.jsonl"),
            Order::Sequential,
            Some(&output_file),
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.failures, 3);
    assert!(output_file.exists());
    let content = fs::read_to_string(&output_file).await.unwrap();
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(lines.len(), 3);
}
