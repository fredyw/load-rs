use load_rs::{HttpMethod, LoadTestRunner, Stats};
use std::fs;

#[tokio::test]
async fn test_output_dir_saves_headers_for_success() {
    let output_path = std::env::current_dir()
        .unwrap()
        .join("tests")
        .join("tmp_output_success");
    if output_path.exists() {
        fs::remove_dir_all(&output_path).unwrap();
    }

    let runner = LoadTestRunner::builder("https://mockhttp.org/get", 1, 1)
        .stats(Stats::All)
        .build()
        .await
        .unwrap();

    runner
        .run(HttpMethod::Get, None, None, Some(&output_path), |_| {})
        .await
        .unwrap();

    let jsonl_path = output_path.join("responses.jsonl");
    assert!(jsonl_path.exists());

    let content = fs::read_to_string(jsonl_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(json.get("headers").is_some());
    assert!(json.get("body").is_some());
    assert!(json.get("status").is_some());
    assert_eq!(json.get("status").unwrap(), 200);

    fs::remove_dir_all(&output_path).unwrap();
}

#[tokio::test]
async fn test_output_dir_saves_headers_for_failure() {
    let output_path = std::env::current_dir()
        .unwrap()
        .join("tests")
        .join("tmp_output_failure");
    if output_path.exists() {
        fs::remove_dir_all(&output_path).unwrap();
    }

    // Using a URL that returns 404
    let runner = LoadTestRunner::builder("https://mockhttp.org/status/404", 1, 1)
        .stats(Stats::All)
        .build()
        .await
        .unwrap();

    runner
        .run(HttpMethod::Get, None, None, Some(&output_path), |_| {})
        .await
        .unwrap();

    let jsonl_path = output_path.join("responses.jsonl");
    assert!(jsonl_path.exists());

    let content = fs::read_to_string(jsonl_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(json.get("headers").is_some());
    assert!(json.get("body").is_some());
    assert!(json.get("status").is_some());
    assert_eq!(json.get("status").unwrap(), 404);

    fs::remove_dir_all(&output_path).unwrap();
}

#[tokio::test]
async fn test_output_dir_save_mode_headers_only() {
    let output_path = std::env::current_dir()
        .unwrap()
        .join("tests")
        .join("tmp_output_headers");
    if output_path.exists() {
        fs::remove_dir_all(&output_path).unwrap();
    }

    let runner = LoadTestRunner::builder("https://mockhttp.org/get", 1, 1)
        .stats(Stats::All)
        .save_mode(load_rs::SaveMode::Headers)
        .build()
        .await
        .unwrap();

    runner
        .run(HttpMethod::Get, None, None, Some(&output_path), |_| {})
        .await
        .unwrap();

    let jsonl_path = output_path.join("responses.jsonl");
    let content = fs::read_to_string(jsonl_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(json.get("headers").is_some());
    assert!(json.get("body").is_none());
    assert!(json.get("status").is_some());

    fs::remove_dir_all(&output_path).unwrap();
}

#[tokio::test]
async fn test_output_dir_manifest_correlation() {
    let output_path = std::env::current_dir()
        .unwrap()
        .join("tests")
        .join("tmp_output_manifest");
    if output_path.exists() {
        fs::remove_dir_all(&output_path).unwrap();
    }

    let manifest_path = std::env::current_dir()
        .unwrap()
        .join("tests")
        .join("test_manifests")
        .join("names.jsonl");

    let runner = LoadTestRunner::builder("https://mockhttp.org", 3, 1)
        .stats(Stats::All)
        .build()
        .await
        .unwrap();

    runner
        .run_from_manifest(
            HttpMethod::Get,
            &manifest_path,
            load_rs::Order::Sequential,
            Some(&output_path),
            |_| {},
        )
        .await
        .unwrap();

    let jsonl_path = output_path.join("responses.jsonl");
    let content = fs::read_to_string(jsonl_path).unwrap();
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(lines.len(), 3);

    let json1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(json1.get("name").unwrap(), "login_request");

    let json2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(json2.get("name").unwrap(), "search_request");

    let json3: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
    assert_eq!(json3.get("name").unwrap(), "line-3");

    fs::remove_dir_all(&output_path).unwrap();
}
