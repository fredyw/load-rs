use load_rs::{LoadTestRunner, Stats};

#[tokio::test]
async fn test_proxy_configuration() {
    // We can at least test that the runner can be initialized with a proxy URL.
    let runner = LoadTestRunner::new(
        "http://localhost:8080",
        1,
        1,
        Stats::All,
        None,
        None,
        None,
        false,
        None,
        None,
        Some("http://proxy.example.com:8080"),
    )
    .await;

    assert!(runner.is_ok());
}

#[tokio::test]
async fn test_invalid_proxy_url() {
    // Garbage like "http://[invalid-ip]" should fail.
    let runner = LoadTestRunner::new(
        "http://localhost:8080",
        1,
        1,
        Stats::All,
        None,
        None,
        None,
        false,
        None,
        None,
        Some("http://[invalid-ip]"),
    )
    .await;

    assert!(runner.is_err());
}
