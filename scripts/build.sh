#!/usr/bin/env bash
set -e

# Fail build, if there are any warnings.
export RUSTFLAGS="-D warnings"

# Generic infrastructure
(
    cd test-stand-infra/protocol
    cargo test --verbose)
(
    cd test-stand-infra/firmware-lib
    cargo test --verbose)
(
    cd test-stand-infra/host-lib
    cargo test --verbose)

# LPC845 test stand
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

# STM32L4 test stand
(
    cd stm32l4-test-stand/test-target
    cargo build --verbose)
(
    cd stm32l4-test-stand/test-assistant
    cargo build --verbose)
(
    cd stm32l4-test-stand/test-suite
    cargo build --tests --verbose)
