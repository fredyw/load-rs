#!/bin/bash

set -ueo pipefail

cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
