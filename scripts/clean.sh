#!/usr/bin/env bash
set -e

# Generic infrastructure
(
    cd test-stand-infra/protocol
    cargo clean)
(
    cd test-stand-infra/firmware-lib
    cargo clean)
(
    cd test-stand-infra/host-lib
    cargo clean)

# LPC845 test stand
(
    cd lpc845-test-stand/messages
    cargo clean)
(
    cd lpc845-test-stand/test-target
    cargo clean)
(
    cd lpc845-test-stand/test-assistant
    cargo clean)
(
    cd lpc845-test-stand/test-suite
    cargo clean)

# STM32L4 test stand
(
    cd stm32l4-test-stand/test-target
    cargo clean)
(
    cd stm32l4-test-stand/test-assistant
    cargo clean)
(
    cd stm32l4-test-stand/test-suite
    cargo clean)
