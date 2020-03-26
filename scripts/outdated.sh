#!/usr/bin/env bash
set -e

(
    cd messages
    cargo outdated --exit-code 1)
(
    cd firmware-lib
    cargo outdated --exit-code 1)
(
    cd host-lib
    cargo outdated --exit-code 1)
(
    cd test-target
    cargo outdated --exit-code 1)
(
    cd test-assistant
    cargo outdated --exit-code 1)
(
    cd test-suite
    cargo outdated --exit-code 1)
