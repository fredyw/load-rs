use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::predicate;
use std::path::PathBuf;

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

    cmd.assert().success().stdout(predicate::str::contains(
        "Sending 5 requests to https://mockhttp.org/get with 2 concurrency",
    ));

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

    cmd.assert().success().stdout(predicate::str::contains(
        "Sending 5 requests to https://mockhttp.org/get with 2 concurrency",
    ));

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

    cmd.assert().success().stdout(predicate::str::contains(
        "Sending 5 requests to https://mockhttp.org/post with 2 concurrency",
    ));

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

    cmd.assert().success().stdout(predicate::str::contains(
        "Sending 5 requests to https://mockhttp.org/put with 2 concurrency",
    ));

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

    cmd.assert().success().stdout(predicate::str::contains(
        "Sending 5 requests to https://mockhttp.org/patch with 2 concurrency",
    ));

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

    cmd.assert().success().stdout(predicate::str::contains(
        "Sending 5 requests to https://mockhttp.org/delete with 2 concurrency",
    ));

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

    cmd.assert().success().stdout(predicate::str::contains(
        "Sending 5 requests to https://mockhttp.org/post with 2 concurrency",
    ));

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

    cmd.assert().success().stdout(predicate::str::contains(
        "Sending 5 requests to https://mockhttp.org/post with 2 concurrency",
    ));

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

    cmd.assert().success().stdout(predicate::str::contains(
        "Sending 5 requests to https://mockhttp.org/post with 2 concurrency",
    ));

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
    let dir = "/tmp/load-rs/cli1";
    let output_dir: PathBuf = dir.into();
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir).unwrap();
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
        dir,
        "https://mockhttp.org/post",
    ]);

    cmd.assert().success();

    assert!(PathBuf::from(format!("{dir}/success-1.json")).exists());
    assert!(PathBuf::from(format!("{dir}/success-2.json")).exists());
    assert!(PathBuf::from(format!("{dir}/success-3.json")).exists());

    Ok(())
}

#[test]
fn run_data_dir_save_responses() -> Result<()> {
    let dir = "/tmp/load-rs/cli2";
    let output_dir: PathBuf = dir.into();
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir).unwrap();
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
        dir,
        "https://mockhttp.org/post",
    ]);

    cmd.assert().success();

    assert!(PathBuf::from(format!("{dir}/success-1-test1.json")).exists());
    assert!(PathBuf::from(format!("{dir}/success-2-test2.json")).exists());
    assert!(PathBuf::from(format!("{dir}/success-3-test3.json")).exists());

    Ok(())
}

#[test]
fn run_manifest_save_responses() -> Result<()> {
    let dir = "/tmp/load-rs/cli3";
    let output_dir: PathBuf = dir.into();
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir).unwrap();
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
        "tests/test_manifest.jsonl",
        "-O",
        "sequential",
        "-o",
        dir,
        "https://mockhttp.org/post",
    ]);

    cmd.assert().success();

    assert!(PathBuf::from(format!("{dir}/success-1.json")).exists());
    assert!(PathBuf::from(format!("{dir}/success-2.json")).exists());
    assert!(PathBuf::from(format!("{dir}/success-3.json")).exists());

    Ok(())
}
