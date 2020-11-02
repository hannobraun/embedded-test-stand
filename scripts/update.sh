#!/usr/bin/env bash
set -e

(
    cd test-stand-infra/protocol
    cargo update)
(
    cd test-stand-infra/firmware-lib
    cargo update)
(
    cd test-stand-infra/host-lib
    cargo update)
(
    cd lpc845-test-stand/messages
    cargo update)
(
    cd lpc845-test-stand/test-target
    cargo update)
(
    cd lpc845-test-stand/test-assistant
    cargo update)
(
    cd lpc845-test-stand/test-suite
    cargo update)
