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
fn run_data_dir() -> Result<()> {
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
