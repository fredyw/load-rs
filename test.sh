#!/bin/bash

set -ueo pipefail

(cd tests/tls; ./gen.sh)
cargo test
