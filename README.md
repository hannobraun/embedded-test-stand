# Embedded Test Stand

## Introduction

A test stand for firmware applications. Allows you to control an application under test while it is running on the hardware, and verify that it behaves correctly.


## Status

As of this writing, this repository contains a test stand that tests various peripheral APIs of LPC8xx HAL, as well as the infrastructure to support it. Work on adding a second test stand is underway.

The existing infrastructure should be able to support test stands for other firmware applications to, but it is still a work in progress and is not as useful as it could be.


## Concepts

This section explains some concepts, which should make the structure of this repository easier to understand.

**Test target**: The subject of the test. The firmware it runs might be part of the system being tested, or it might be purpose-built to support the test. In both cases it communicates with the host system, so that the test suite running there can trigger behavior and check results.

**Test assistant**: A development board that assists the test suite running on the host in performing the testing. It provides the test suite with additional capabilities that the host system might not have otherwise (i.e. GPIO, protocols like I2C/SPI).

**Test node**: The umbrella term that can refer to either the test target or the test assistant.

**Test suite**: A collection of test cases, which run on the host computer. It communicates with the test nodes, to orchestrate the test and gather information about the test target's behavior.

**Test case**: A single test that is part of a test suite.


## Structure

The crates in this repository are split into two groups: Infrastructure that can be used to build target-specific test stands, and the target-specific test stands.

### Test Stand Infrastructure

These are the crates that are independent of any specific test suite. If you want to use this test stand for your own project, these are the crates you want to use:

- `test-stand-infra/protocol`: Building blocks that can be used to build a protocol for communication between the host and the test nodes.
- `test-stand-infra/firmware-lib`: Library for firmware running on the target or assistant. This might be deprecated in the future. See issue [#85](https://github.com/braun-embedded/lpc845-test-stand/issues/85).
- `host-lib`: Library that provides functionality for test suites running on the host.

### LPC845 Test Stand

Supports a test suite that covers some of the peripheral APIs in the LPC8xx HAL library. See [its own README file](https://github.com/braun-embedded/embedded-test-stand/blob/master/lpc845-test-stand/README.md) for more information.


## License

Code in this repository, unless specifically noted otherwise, is available under the terms under the [0BSD License]. This essentially means you can do what you want with it, without any restrictions.

See [LICENSE.md] for the full license text.

[0BSD License]: https://opensource.org/licenses/0BSD
[LICENSE.md]: LICENSE.md

**Created by [Braun Embedded](https://braun-embedded.com/)** <br />
**Initial development sponsored by [Georg Fischer Signet](http://www.gfsignet.com/)**
