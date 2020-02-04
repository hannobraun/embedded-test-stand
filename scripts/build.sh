#!/usr/bin/env bash
set -e

# Fail build, if there are any warnings.
export RUSTFLAGS="-D warnings"

(
    cd test-lib
    cargo test --verbose --features=firmware)
(
    cd test-lib
    cargo test --verbose --features=host)
(
    cd test-firmware
    cargo build --verbose)
(
    cd test-suite
    cargo build --tests --verbose)
