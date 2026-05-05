use load_rs::{HttpMethod, LoadTestRunner, Stats};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

mod common;
use common::*;

#[tokio::test]
async fn test_ui_debouncing() {
    let server = run_perf_server().await.unwrap();
    let url = format!("http://{}", server.addr);
    let requests = 20;
    let concurrency = 5;
    let runner = LoadTestRunner::builder(&url, requests, concurrency)
        .stats(Stats::All)
        .build()
        .await
        .unwrap();
    let callback_count = Arc::new(AtomicU32::new(0));
    let cb = Arc::clone(&callback_count);
    let result = runner
        .run(HttpMethod::Get, None, None, None, move |event| {
            if let load_rs::LoadTestEvent::ProgressUpdate(_) = event {
                cb.fetch_add(1, Ordering::SeqCst);
            }
        })
        .await
        .unwrap();

    assert_eq!(result.completed, requests);

    let count = callback_count.load(Ordering::SeqCst);
    // With 20 requests and 50ms delay each, total time is roughly (20/5)*50ms = 200ms.
    // With 100ms debouncing, we expect roughly 2-4 calls.
    // Definitely less than 20.
    assert!(
        count < requests,
        "Callback count {} should be less than total requests {}",
        count,
        requests
    );
    assert!(
        count >= 1,
        "Callback should be called at least once at the end"
    );
}
