#!/usr/bin/env bash
set -e

(
    cd messages
    cargo update)
(
    cd firmware-lib
    cargo update)
(
    cd host-lib
    cargo update)
(
    cd test-target
    cargo update)
(
    cd test-assistant
    cargo update)
(
    cd test-suite
    cargo update)
