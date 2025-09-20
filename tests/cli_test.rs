use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::predicate;

#[test]
fn test_get() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args(&[
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
fn test_head() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args(&[
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
fn test_post() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args(&[
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
fn test_put() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args(&[
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
fn test_patch() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args(&[
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
fn test_delete() -> Result<()> {
    let mut cmd = Command::cargo_bin("load-rs")?;
    cmd.args(&[
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
