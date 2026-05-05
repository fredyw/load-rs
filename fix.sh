#!/bin/bash

set -ueo pipefail

cargo fmt
cargo fix --allow-dirty --all-targets --all-features
cargo clippy --fix --allow-dirty --all-targets --all-features
