use load_rs::{LoadTestRunner, Stats};

#[tokio::test]
async fn test_proxy_configuration() {
    // We can at least test that the runner can be initialized with a proxy URL.
    let runner = LoadTestRunner::builder("http://localhost:8080", 1, 1)
        .stats(Stats::All)
        .proxy("http://proxy.example.com:8080")
        .build()
        .await;

    assert!(runner.is_ok());
}

#[tokio::test]
async fn test_invalid_proxy_url() {
    // Garbage like "http://[invalid-ip]" should fail.
    let runner = LoadTestRunner::builder("http://localhost:8080", 1, 1)
        .stats(Stats::All)
        .proxy("http://[invalid-ip]")
        .build()
        .await;

    assert!(runner.is_err());
}
