# load-rs

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![load-rs](https://github.com/fredyw/load-rs/actions/workflows/load-rs.yml/badge.svg)](https://github.com/fredyw/load-rs/actions/workflows/load-rs.yml)

A simple load testing tool written in Rust.

![Demo](demo.gif)

### Usage

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
  -I, --insecure <INSECURE>            Allows insecure connections by skipping TLS certificate verification [possible values: true, false]
  -O, --order <ORDER>                  Order to process files from --data-dir [default: sequential]
  -o, --output-dir <OUTPUT_DIR>        Directory to save responses to
  -G, --debug                          Performs a single request and dumps the response
  -h, --help                           Print help
  -V, --version                        Print version
```

### Examples

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

## Building

To build the project, you need to have Rust installed. You can install it from [here](https://www.rust-lang.org/tools/install).

Once you have Rust installed, you can build the project by running the following command:

```
./build.sh --release
```

The binary will be located in `target/release/load-rs`.

## Testing

To run the tests, you can use the following command:

```
./test.sh
```

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue.

## License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for details.
