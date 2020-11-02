#!/usr/bin/env bash
set -e

(
    cd test-stand-infra/protocol
    cargo upgrades)
(
    cd test-stand-infra/firmware-lib
    cargo upgrades)
(
    cd test-stand-infra/host-lib
    cargo upgrades)
(
    cd lpc845-test-stand/messages
    cargo upgrades)
(
    cd lpc845-test-stand/test-target
    cargo upgrades)
(
    cd lpc845-test-stand/test-assistant
    cargo upgrades)
(
    cd lpc845-test-stand/test-suite
    cargo upgrades)
