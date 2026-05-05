use load_rs::Body::Data;
use load_rs::{HttpMethod, LoadTestRunner, Order, Stats};
use reqwest::header::HeaderMap;
use std::path::{Path, PathBuf};
use tokio::fs;

mod common;

#[tokio::test]
async fn run_success_save_responses() {
    let dir = "/tmp/load-rs/lib1";
    let output_dir: PathBuf = dir.into();
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).await.unwrap();
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
            Some(&output_dir),
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 3);
    assert!(PathBuf::from(format!("{dir}/success-1.json")).exists());
    assert!(PathBuf::from(format!("{dir}/success-2.json")).exists());
    assert!(PathBuf::from(format!("{dir}/success-3.json")).exists());
}

#[tokio::test]
async fn run_failure_save_responses() {
    let dir = "/tmp/load-rs/lib2";
    let output_dir: PathBuf = dir.into();
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).await.unwrap();
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
            Some(&output_dir),
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.failures, 3);
    assert!(PathBuf::from(format!("{dir}/failure-1.json")).exists());
    assert!(PathBuf::from(format!("{dir}/failure-2.json")).exists());
    assert!(PathBuf::from(format!("{dir}/failure-3.json")).exists());
}

#[tokio::test]
async fn run_from_dir_success_save_responses() {
    let dir = "/tmp/load-rs/lib3";
    let output_dir: PathBuf = dir.into();
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).await.unwrap();
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
            Some(&output_dir),
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 3);
    assert!(PathBuf::from(format!("{dir}/success-1-test1.json")).exists());
    assert!(PathBuf::from(format!("{dir}/success-2-test2.json")).exists());
    assert!(PathBuf::from(format!("{dir}/success-3-test3.json")).exists());
}

#[tokio::test]
async fn run_from_dir_failure_save_responses() {
    let dir = "/tmp/load-rs/lib4";
    let output_dir: PathBuf = dir.into();
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).await.unwrap();
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
            Some(&output_dir),
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.failures, 3);
    assert!(PathBuf::from(format!("{dir}/failure-1-test1.json")).exists());
    assert!(PathBuf::from(format!("{dir}/failure-2-test2.json")).exists());
    assert!(PathBuf::from(format!("{dir}/failure-3-test3.json")).exists());
}

#[tokio::test]
async fn run_from_manifest_success_save_responses() {
    let dir = "/tmp/load-rs/lib5";
    let output_dir: PathBuf = dir.into();
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).await.unwrap();
    }

    let runner = LoadTestRunner::builder("https://mockhttp.org/post", 3, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let result = runner
        .run_from_manifest(
            HttpMethod::Post,
            Path::new("tests/test_manifests/manifest1.jsonl"),
            Order::Sequential,
            Some(&output_dir),
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.success, 3);
    assert!(PathBuf::from(format!("{dir}/success-1.json")).exists());
    assert!(PathBuf::from(format!("{dir}/success-2.json")).exists());
    assert!(PathBuf::from(format!("{dir}/success-3.json")).exists());
}

#[tokio::test]
async fn run_from_manifest_failure_save_responses() {
    let dir = "/tmp/load-rs/lib6";
    let output_dir: PathBuf = dir.into();
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).await.unwrap();
    }

    let runner = LoadTestRunner::builder("https://mockhttp.org/get", 3, 2)
        .stats(Stats::Success)
        .build()
        .await
        .unwrap();

    let result = runner
        .run_from_manifest(
            HttpMethod::Post,
            Path::new("tests/test_manifests/manifest1.jsonl"),
            Order::Sequential,
            Some(&output_dir),
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.failures, 3);
    assert!(PathBuf::from(format!("{dir}/failure-1.json")).exists());
    assert!(PathBuf::from(format!("{dir}/failure-2.json")).exists());
    assert!(PathBuf::from(format!("{dir}/failure-3.json")).exists());
}
