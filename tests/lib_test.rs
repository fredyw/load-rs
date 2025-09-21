extern crate load_rs;

use load_rs::Body::{Data, DataFile};
use load_rs::{HttpMethod, LoadTestRunner, Order};
use reqwest::header::HeaderMap;
use std::path::PathBuf;
use tokio::fs;

#[tokio::test]
async fn run_get() {
    let runner = LoadTestRunner::new("https://mockhttp.org/get", 5, 2, &None, &None, &None, &None)
        .await
        .unwrap();

    let result = runner
        .run(HttpMethod::Get, None, None, &None, |_| {})
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
        .run(HttpMethod::Head, None, None, &None, |_| {})
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
            &None,
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
            &None,
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
            &None,
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
            &None,
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
            Order::Sequential,
            &None,
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
            Some(DataFile("tests/test_requests/test1.json".into())),
            &None,
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
            &None,
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
            Order::Sequential,
            &None,
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
            Order::Sequential,
            &None,
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
            &None,
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
            &None,
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
async fn debug_get() {
    let runner = LoadTestRunner::new("https://mockhttp.org/get", 5, 2, &None, &None, &None, &None)
        .await
        .unwrap();

    let response = runner.debug(HttpMethod::Get, None, None).await.unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn debug_head() {
    let runner = LoadTestRunner::new("https://mockhttp.org/get", 5, 2, &None, &None, &None, &None)
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
        &None,
        &None,
        &None,
        &None,
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
    let runner = LoadTestRunner::new("https://mockhttp.org/put", 5, 2, &None, &None, &None, &None)
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
        &None,
        &None,
        &None,
        &None,
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
        &None,
        &None,
        &None,
        &None,
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
        &None,
        &None,
        &None,
        &None,
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
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let response = runner
        .debug_from_dir(
            HttpMethod::Post,
            Some(headers),
            &"tests/test_requests".into(),
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
        &None,
        &None,
        &None,
        &None,
    )
    .await
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let response = runner
        .debug_from_dir(
            HttpMethod::Post,
            Some(headers),
            &"tests/test_requests".into(),
            Order::Random,
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn run_success_save_responses() {
    let dir = "/tmp/load-rs/lib1";
    let output_dir: PathBuf = dir.into();
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).await.unwrap();
    }

    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        3,
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
            &Some(output_dir),
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

    let runner = LoadTestRunner::new("https://mockhttp.org/get", 3, 2, &None, &None, &None, &None)
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run(
            HttpMethod::Post,
            Some(headers),
            Some(Data("{\"message\": \"hello\"}".into())),
            &Some(output_dir),
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

    let runner = LoadTestRunner::new(
        "https://mockhttp.org/post",
        3,
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
            Order::Sequential,
            &Some(output_dir),
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

    let runner = LoadTestRunner::new("https://mockhttp.org/get", 3, 2, &None, &None, &None, &None)
        .await
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let result = runner
        .run_from_dir(
            HttpMethod::Post,
            Some(headers),
            &"tests/test_requests".into(),
            Order::Sequential,
            &Some(output_dir),
            |_| {},
        )
        .await
        .unwrap();

    assert_eq!(result.failures, 3);
    assert!(PathBuf::from(format!("{dir}/failure-1-test1.json")).exists());
    assert!(PathBuf::from(format!("{dir}/failure-2-test2.json")).exists());
    assert!(PathBuf::from(format!("{dir}/failure-3-test3.json")).exists());
}
