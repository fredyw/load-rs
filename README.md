# load-rs

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Build & Test](https://github.com/fredyw/load-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/fredyw/load-rs/actions/workflows/rust.yml)


A simple load testing tool written in Rust.

## Usage
```
Usage: load-rs [OPTIONS] --requests <REQUESTS> --concurrency <CONCURRENCY> <URL>

Arguments:
  <URL>  Target URL to send requests to

Options:
  -n, --requests <REQUESTS>        Total number of requests to send
  -c, --concurrency <CONCURRENCY>  Number of concurrent requests to run at a time
  -X, --method <METHOD>            HTTP method to use for the requests [default: get]
  -H, --header <HEADER>            Custom HTTP header(s) in "key: value" format. Can be repeated
  -d, --data <DATA>                Request body as a string
  -D, --data-file <DATA_FILE>      File to read the request body from
  -i, --data-dir <DATA_DIR>        Directory of files to use as request bodies
  -C, --cacert <CA_CERT>           Custom CA certificate file (PEM format)
  -E, --cert <CERT>                Public certificate file (PEM format)
  -k, --key <KEY>                  Private key file (PEM format)
  -I, --insecure                   Allows insecure connections by skipping TLS certificate verification
  -O, --order <ORDER>              Order to process files from --data-dir [default: sequential]
  -h, --help                       Print help
  -V, --version                    Print versio
```
