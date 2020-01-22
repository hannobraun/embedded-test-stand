#!/usr/bin/env bash
set -e

# Fail build, if there are any warnings.
export RUSTFLAGS="-D warnings"

(
    cd test-firmware
    cargo build --verbose)

(
    cd test-suite
    cargo build --tests --verbose)
