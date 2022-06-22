#!/usr/bin/env bash
set -euo pipefail

cargo +nightly fmt
cargo clippy
cargo nextest run
