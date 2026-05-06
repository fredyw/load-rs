use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::predicate;

#[test]
fn run_get() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "-n",
        "5",
        "-c",
        "2",
        "-X",
        "GET",
        "https://mockhttp.org/get",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Overview:"));

    Ok(())
}

#[test]
fn run_head() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "-n",
        "5",
        "-c",
        "2",
        "-X",
        "HEAD",
        "https://mockhttp.org/get",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Overview:"));

    Ok(())
}

#[test]
fn run_post() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "-n",
        "5",
        "-c",
        "2",
        "-X",
        "POST",
        "-H",
        "Content-Type: application/json",
        "-d",
        "{\"message\":\"Hello, world!\"}",
        "https://mockhttp.org/post",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Overview:"));

    Ok(())
}

#[test]
fn run_put() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "-n",
        "5",
        "-c",
        "2",
        "-X",
        "PUT",
        "-H",
        "Content-Type: application/json",
        "-d",
        "{\"message\":\"Hello, world!\"}",
        "https://mockhttp.org/put",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Overview:"));

    Ok(())
}

#[test]
fn run_patch() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "-n",
        "5",
        "-c",
        "2",
        "-X",
        "PATCH",
        "-H",
        "Content-Type: application/json",
        "-d",
        "{\"message\":\"Hello, world!\"}",
        "https://mockhttp.org/patch",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Overview:"));

    Ok(())
}

#[test]
fn run_delete() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "-n",
        "5",
        "-c",
        "2",
        "-X",
        "DELETE",
        "-H",
        "Content-Type: application/json",
        "-d",
        "{\"message\":\"Hello, world!\"}",
        "https://mockhttp.org/delete",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Overview:"));

    Ok(())
}

#[test]
fn run_data_file() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "-n",
        "5",
        "-c",
        "2",
        "-X",
        "POST",
        "-H",
        "Content-Type: application/json",
        "-D",
        "tests/test_requests/test1.json",
        "https://mockhttp.org/post",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Overview:"));

    Ok(())
}

#[test]
fn run_data_dir_sequential() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "-n",
        "5",
        "-c",
        "2",
        "-X",
        "POST",
        "-H",
        "Content-Type: application/json",
        "-i",
        "tests/test_requests",
        "-O",
        "sequential",
        "https://mockhttp.org/post",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Overview:"));

    Ok(())
}

#[test]
fn run_data_dir_random() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "-n",
        "5",
        "-c",
        "2",
        "-X",
        "POST",
        "-H",
        "Content-Type: application/json",
        "-i",
        "tests/test_requests",
        "-O",
        "random",
        "https://mockhttp.org/post",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Overview:"));

    Ok(())
}

#[test]
fn debug_get() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "--debug",
        "-n",
        "5",
        "-c",
        "2",
        "-X",
        "GET",
        "https://mockhttp.org/get",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("HTTP/2.0 200 OK"));

    Ok(())
}

#[test]
fn debug_minimal_args() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args(["--debug", "https://mockhttp.org/get"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("HTTP/2.0 200 OK"));

    Ok(())
}

#[test]
fn debug_head() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "--debug",
        "-n",
        "5",
        "-c",
        "2",
        "-X",
        "HEAD",
        "https://mockhttp.org/get",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("HTTP/2.0 200 OK"));

    Ok(())
}

#[test]
fn debug_post() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "--debug",
        "-n",
        "5",
        "-c",
        "2",
        "-X",
        "POST",
        "-H",
        "Content-Type: application/json",
        "-d",
        "{\"message\":\"Hello, world!\"}",
        "https://mockhttp.org/post",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("HTTP/2.0 200 OK"));

    Ok(())
}

#[test]
fn debug_put() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "--debug",
        "-n",
        "5",
        "-c",
        "2",
        "-X",
        "PUT",
        "-H",
        "Content-Type: application/json",
        "-d",
        "{\"message\":\"Hello, world!\"}",
        "https://mockhttp.org/put",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("HTTP/2.0 200 OK"));

    Ok(())
}

#[test]
fn debug_patch() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "--debug",
        "-n",
        "5",
        "-c",
        "2",
        "-X",
        "PATCH",
        "-H",
        "Content-Type: application/json",
        "-d",
        "{\"message\":\"Hello, world!\"}",
        "https://mockhttp.org/patch",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("HTTP/2.0 200 OK"));

    Ok(())
}

#[test]
fn debug_delete() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "--debug",
        "-n",
        "5",
        "-c",
        "2",
        "-X",
        "DELETE",
        "-H",
        "Content-Type: application/json",
        "-d",
        "{\"message\":\"Hello, world!\"}",
        "https://mockhttp.org/delete",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("HTTP/2.0 200 OK"));

    Ok(())
}

#[test]
fn debug_data_file() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "--debug",
        "-n",
        "5",
        "-c",
        "2",
        "-X",
        "POST",
        "-H",
        "Content-Type: application/json",
        "-D",
        "tests/test_requests/test1.json",
        "https://mockhttp.org/post",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("HTTP/2.0 200 OK"));

    Ok(())
}

#[test]
fn debug_data_dir_sequential() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "--debug",
        "-n",
        "5",
        "-c",
        "2",
        "-X",
        "POST",
        "-H",
        "Content-Type: application/json",
        "-i",
        "tests/test_requests",
        "-O",
        "sequential",
        "https://mockhttp.org/post",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("HTTP/2.0 200 OK"));

    Ok(())
}

#[test]
fn debug_data_dir_random() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "--debug",
        "-n",
        "5",
        "-c",
        "2",
        "-X",
        "POST",
        "-H",
        "Content-Type: application/json",
        "-i",
        "tests/test_requests",
        "-O",
        "random",
        "https://mockhttp.org/post",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("HTTP/2.0 200 OK"));

    Ok(())
}

#[test]
fn run_save_responses() -> Result<()> {
    let output_file = std::env::temp_dir().join("load-rs-cli1.jsonl");
    if output_file.exists() {
        std::fs::remove_file(&output_file).unwrap();
    }
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "-n",
        "3",
        "-c",
        "2",
        "-X",
        "POST",
        "-H",
        "Content-Type: application/json",
        "-d",
        "{\"message\":\"Hello, world!\"}",
        "-o",
        output_file.to_str().unwrap(),
        "https://mockhttp.org/post",
    ]);

    cmd.assert().success();

    assert!(output_file.exists());
    let content = std::fs::read_to_string(&output_file).unwrap();
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(lines.len(), 3);

    Ok(())
}

#[test]
fn run_data_dir_save_responses() -> Result<()> {
    let output_file = std::env::temp_dir().join("load-rs-cli2.jsonl");
    if output_file.exists() {
        std::fs::remove_file(&output_file).unwrap();
    }
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "-n",
        "3",
        "-c",
        "2",
        "-X",
        "POST",
        "-H",
        "Content-Type: application/json",
        "-i",
        "tests/test_requests",
        "-O",
        "sequential",
        "-o",
        output_file.to_str().unwrap(),
        "https://mockhttp.org/post",
    ]);

    cmd.assert().success();

    assert!(output_file.exists());
    let content = std::fs::read_to_string(&output_file).unwrap();
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(lines.len(), 3);

    Ok(())
}

#[test]
fn run_manifest_save_responses() -> Result<()> {
    let output_file = std::env::temp_dir().join("load-rs-cli3.jsonl");
    if output_file.exists() {
        std::fs::remove_file(&output_file).unwrap();
    }
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "-n",
        "3",
        "-c",
        "2",
        "-X",
        "POST",
        "-m",
        "tests/test_manifests/requests.jsonl",
        "-O",
        "sequential",
        "-o",
        output_file.to_str().unwrap(),
        "https://mockhttp.org/post",
    ]);

    cmd.assert().success();

    assert!(output_file.exists());
    let content = std::fs::read_to_string(&output_file).unwrap();
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(lines.len(), 3);

    Ok(())
}

#[test]
fn run_unit_seconds() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "-n",
        "1",
        "-c",
        "1",
        "-X",
        "GET",
        "-u",
        "seconds",
        "https://mockhttp.org/get",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Duration:"))
        .stdout(predicate::str::is_match(r"Duration:.*\d+\.\d{2}s.*")?);

    Ok(())
}

#[test]
fn run_json_output() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "-n",
        "1",
        "-c",
        "1",
        "-X",
        "GET",
        "--json",
        "https://mockhttp.org/get",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"success\": 1"))
        .stdout(predicate::str::contains("\"completed\": 1"));

    Ok(())
}

#[test]
fn run_duration_based() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "-z",
        "2s",
        "-c",
        "2",
        "-X",
        "GET",
        "https://mockhttp.org/get",
    ]);

    let now = std::time::Instant::now();
    cmd.assert().success();
    let elapsed = now.elapsed();

    // Should take around 2 seconds (plus some overhead)
    assert!(elapsed >= std::time::Duration::from_secs(2));

    Ok(())
}

#[test]
fn run_rate_limited() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args([
        "-n",
        "10",
        "-c",
        "2",
        "-r",
        "5",
        "-X",
        "GET",
        "https://mockhttp.org/get",
    ]);

    let now = std::time::Instant::now();
    cmd.assert().success();
    let elapsed = now.elapsed();

    // 10 requests at 5 RPS should take at least 2 seconds
    assert!(elapsed >= std::time::Duration::from_secs(2));

    Ok(())
}
