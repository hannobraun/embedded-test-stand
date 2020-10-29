#!/usr/bin/env bash
set -e

# Fail build, if there are any warnings.
export RUSTFLAGS="-D warnings"

(
    cd test-stand-infra/protocol
    cargo test --verbose)
(
    cd test-stand-infra/firmware-lib
    cargo test --verbose)
(
    cd test-stand-infra/host-lib
    cargo test --verbose)
(
    cd lpc845-test-stand/messages
    cargo test --verbose)
(
    cd lpc845-test-stand/test-target
    cargo build --verbose)
(
    cd lpc845-test-stand/test-assistant
    cargo build --verbose)
(
    cd lpc845-test-stand/test-suite
    cargo build --tests --verbose)
