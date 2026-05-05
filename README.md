# load-rs

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![CI](https://github.com/fredyw/load-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/fredyw/load-rs/actions/workflows/ci.yml)

A simple HTTP load testing library and CLI tool written in Rust.

![Demo](demo.gif)

## Table of Contents

- [Usage](#usage)
  - [Command Line Options](#command-line-options)
  - [Output Files](#output-files)
  - [Request Manifest](#request-manifest)
  - [Order](#order)
  - [TLS](#tls)
  - [Statistics](#statistics)
  - [Timeout](#timeout)
  - [User Agent](#user-agent)
  - [Proxy](#proxy)
  - [Unit](#unit)
  - [Debugging](#debugging)
  - [Examples](#examples)
- [Building](#building)
- [Installing](#installing)
- [Testing](#testing)
- [Library Usage](#library-usage)
  - [Basic Usage](#basic-usage)
  - [Custom Request Generator](#custom-request-generator)
- [Contributing](#contributing)
- [License](#license)

### Usage

#### Command Line Options

```
Usage: load-rs [OPTIONS] --requests <REQUESTS> --concurrency <CONCURRENCY> <URL>

Arguments:
  <URL>  Target URL to send requests to

Options:
  -n, --requests <REQUESTS>            Total number of requests to send
  -c, --concurrency <CONCURRENCY>      Number of concurrent requests to run at a time
  -X, --method <METHOD>                HTTP method to use for the requests [default: get]
  -H, --header <HEADER>                Custom HTTP header(s) in "key: value" format. Can be repeated
  -d, --data <DATA>                    Request body as a string
  -D, --data-file <DATA_FILE>          File to read the request body from
  -i, --data-dir <DATA_DIR>            Directory of files to use as request bodies
  -m, --manifest-file <MANIFEST_FILE>  Request manifest file (JSON Lines format)
  -C, --cacert <CA_CERT>               Custom CA certificate file (PEM format)
  -E, --cert <CERT>                    Public certificate file (PEM format)
  -k, --key <KEY>                      Private key file (PEM format)
  -I, --insecure                       Allows insecure connections by skipping TLS certificate verification
  -O, --order <ORDER>                  Order to process files from --data-dir or --manifest-file [default: sequential]
  -o, --output-dir <OUTPUT_DIR>        Directory to save responses to
  -G, --debug                          Performs a single request and dumps the response
  -s, --stats <STATS>                  Specifies which requests to include in the statistics [default: all]
  -t, --timeout <TIMEOUT>              Request timeout in seconds
  -A, --user-agent <USER_AGENT>        Custom user agent
  -p, --proxy <PROXY>                  Proxy server URL
  -u, --unit <UNIT>                    Unit of measurement (seconds or milliseconds) [default: milliseconds]
  -q, --quiet                          Quiet mode: suppress progress updates
  -h, --help                           Print help
  -V, --version                        Print version
```

#### Output Files

When the `-o` or `--output-dir` option is specified, `load-rs` will save the response of each request
to a file in the specified directory.

The output file is a JSON object with the following fields:

- `version`: The HTTP version of the response.
- `status`: The HTTP status code of the response.
- `headers`: A map of the response headers.
- `body`: The response body as a string. If the body is not valid UTF-8, it will be base64-encoded.
- `duration`: The duration of the request.
- `error`: The error message if the request failed.

#### Request Manifest

The manifest file is a [JSON Lines](https://jsonlines.org/) file where each line is a JSON object
that defines a request. The following fields are supported:

- `headers`: A map of HTTP headers to be sent with the request.
- `body`: The request body as a string.
- `binary_body`: The request body as a base64-encoded string.

**Note:** If both `body` and `binary_body` are specified, `body` will be used.

**Example `manifest.jsonl`**

```json
{"headers": {"Content-Type": "application/json"}, "body": "{\"key\": \"value1\"}"}
{"headers": {"Content-Type": "application/json"}, "body": "{\"key\": \"value2\"}"}
{"headers": {"Content-Type": "application/octet-stream"}, "binary_body": "SGVsbG8gd29ybGQ="}
```

#### Order

The `-O` or `--order` option allows you to control the order in which requests are sent when using
either the `--data-dir` or `--manifest-file` option. The following values are supported:

- `sequential` (default): Requests are sent in the order they appear in the directory or manifest file.
- `random`: Requests are sent in a random order.

#### TLS

`load-rs` supports several options for configuring TLS:

- `-C, --cacert <CA_CERT>`: Use a custom CA certificate file (PEM format) to verify the server's certificate.
- `-E, --cert <CERT>`: Use a client certificate file (PEM format) for mutual TLS authentication.
- `-k, --key <KEY>`: Use a private key file (PEM format) for the client certificate.
- `-I, --insecure`: Allows insecure connections by skipping TLS certificate verification.

#### Statistics

The `s` or `--stats` option allows you to control which requests are included in the statistics. The following
values are supported:

- `success`: Only include successful requests in the statistics.
- `error`: Only include failed requests in the statistics.
- `all` (default): Include all requests (successful and failed) in the statistics.

#### Timeout

The `-t` or `--timeout` option allows you to set a timeout in seconds for each request. By default, requests do not time out.

#### User Agent

The `-A` or `--user-agent` option allows you to set a custom user agent string for each request. By default, requests do not include a custom user agent.

#### Proxy

The `-p` or `--proxy` option allows you to route requests through a proxy server.

#### Unit

The `-u` or `--unit` option allows you to configure the unit of measurement used in the output for duration and latencies. Supported values are `seconds` and `milliseconds`. By default, it is in `milliseconds`.

#### Quiet Mode

The `-q` or `--quiet` option suppresses the real-time progress bar and updates during the test. This can slightly improve performance by reducing terminal I/O and local CPU overhead, and is useful for automation or when redirecting output to a file.

#### Debugging

The `-G` or `--debug` option can be used to perform a single request and dump the response to the console.
This is useful for verifying that your requests are correct and that the server is responding as expected.
When using this option, the `-n`, `-c`, and `-o` options are ignored.

#### Examples

**GET request**

```
load-rs -n 100 -c 10 http://localhost:8080
```

**POST request with a JSON body**

```
load-rs -n 100 -c 10 -X POST -d '{"key": "value"}' http://localhost:8080
```

**POST request with a body from a file**

```
load-rs -n 100 -c 10 -X POST -D /path/to/body.json http://localhost:8080
```

**POST request with bodies from a directory**

```
load-rs -n 100 -c 10 -X POST -i /path/to/bodies http://localhost:8080
```

**POST request with a manifest file**

```
load-rs -n 100 -c 10 -X POST -m /path/to/manifest.jsonl http://localhost:8080
```

### Building

To build the project, you need to have Rust installed. You can install it from [here](https://www.rust-lang.org/tools/install).

Once you have Rust installed, you can build the project by running the following command:

```
./build.sh --release
```

The binary will be located in `target/release/load-rs`.

### Installing

To install `load-rs`, you can use the following command.

```
./install.sh
```

### Testing

To run the tests, you can use the following command.

```
./test.sh
```

### Library Usage

`load-rs` can be used as a library in your own Rust projects. Add it to your `Cargo.toml`:

```toml
[dependencies]
load-rs = { git = "https://github.com/fredyw/load-rs.git" }
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

#### Basic Usage

The following example shows how to run a simple GET load test:

```rust
use load_rs::{LoadTestRunner, HttpMethod, Stats, LoadTestEvent};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = "http://localhost:8080";
    let requests = 100;
    let concurrency = 10;

    // Initialize the runner using the builder pattern
    let runner = LoadTestRunner::builder(url, requests, concurrency)
        .stats(Stats::All)
        .timeout(30)
        .build()
        .await?;

    // Run the test
    let result = runner.run(
        HttpMethod::Get,
        None, // headers
        None, // body
        None, // output_dir
        |event| {
            // Handle real-time events
            match event {
                LoadTestEvent::ProgressUpdate(stats) => {
                    println!("Progress: {}/{} requests, RPS: {:.2}", 
                        stats.completed, requests, stats.rps);
                }
                LoadTestEvent::RequestFinished(res) => {
                    if !res.success {
                        eprintln!("Request {} failed: {:?}", res.iteration, res.error);
                    }
                }
            }
        }
    ).await?;

    println!("\nTest Complete!");
    println!("Average Latency: {:?}", result.avg);
    println!("P95 Latency: {:?}", result.p95);

    Ok(())
}
```

#### Custom Request Generator

For more complex scenarios, you can use a custom request generator to dynamically create requests (e.g., with unique IDs or timestamps in the body).

```rust
use load_rs::{LoadTestRunner, Stats};
use reqwest::{Client, Method};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runner = LoadTestRunner::builder("http://localhost:8080", 1000, 50)
        .stats(Stats::All)
        .build()
        .await?;

    // You can share a client or any other state in the generator
    let client = Client::new();

    let result = runner.run_with_generator(
        move |iteration| {
            // This closure is called for each request
            let req = client.request(Method::POST, "http://localhost:8080/ingest")
                .header("X-Iteration", iteration.to_string())
                .body(format!("{{\"id\": {}, \"timestamp\": {}}}", 
                    iteration, 
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs()))
                .build()?;
            
            // Return the request and an optional filename for saving the response
            Ok((req, None))
        },
        None, // output_dir
        |_| {} // event callback
    ).await?;

    println!("Total Success: {}", result.success);
    Ok(())
}
```

### Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue.

If you are an AI agent, please refer to [AGENTS.md](AGENTS.md) for specific guidelines.


### License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for details.
