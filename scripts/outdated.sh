#!/usr/bin/env bash
set -e

(
    cd protocol
    cargo upgrades)
(
    cd messages
    cargo upgrades)
(
    cd firmware-lib
    cargo upgrades)
(
    cd host-lib
    cargo upgrades)
(
    cd test-target
    cargo upgrades)
(
    cd test-assistant
    cargo upgrades)
(
    cd test-suite
    cargo upgrades)
