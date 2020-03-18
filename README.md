# LPC845 Test Stand

## Introduction

A test stand for firmware applications. Allows you to control an application under test while it is running on the hardware, and verify that it behaves correctly.

While it is desirable to make this project available for all hardware eventually, it only supports NXP LPC845 microcontrollers right now, to make the initial development easier.


## Status

Implementation work has started, and a small (but growing) test suite covering LPC8xx HAL APIs is available. Work is being done to extend this test suite, and in doing so, make the infrastructure supporting it more useful and ready for real-world use.

No work is being done right now to support other microcontrollers than the LPC845. This would be a welcome addition though, so please get in touch by opening an issue, if you're interested.


## Concepts

This section explains some concepts, which should make the structure of this repository easier to understand.

**Test target**: The test target is the hardware under test. It is a microcontroller that runs special firmware, which can communicate with the test suite.

**Test suite**: A collection of test cases, which runs on the host computer. It communicates with the firmwares on the test target and test assistant, to orchestrate the test and gather information about the test target's behavior.

**Test case**: A single test that is part of a test suite.

**Test assistant**: Additional hardware (likely a microcontroller development board) that assists the test suite in orchestrating the tests, and gathering information about the test target's behavior.


## Structure

At the time of writing, there are 5 Cargo packages in this repository. It's a bit unfortunate that there are this many, but it doesn't make sense to reduce the number, because of the following factors:

1. There is firmware code targetting microcontrollers and code intended to be run on the host computer. Those have very different requirements regarding their dependencies, which makes having them in the same crate more trouble than it would be worth.
1. Right now, there are basically two projects in this repository: The test framework, and the test suite for LPC845 HAL. Eventually, both can live in separate places, but for now it makes sense to develop them together.

### Test Framework Crates

These are the crates that are independent of the LPC845 test suite. If you want to use this test stand for your own project, these are the crates you want to use:

- `firmware-lib`: Library for firmware running on the target or assistant.
- `host-lib`: Library for test suites running on the host.

### LPC845 Test Suite

These crates belong to the LPC845 test suite. They are not directly applicable to other uses, except to serve as an example:

- `messages`: The messages used to communicate between test suite and target firmware.
- `test-target`: The firmware running on the hardware under test.
- `test-assistant`: The firmware running on the test assistant.
- `test-suite`: The test suite itself, plus some suite-specific convenience wrappers around APIs in `host-lib`.


## Running the Test Suite

This repository contains an example test suite that sends commands to the device under test and verifies its response.

Before you can successfully run the test suite, you need the following:

1. Two [LPC845-BRK] boards. One will be the test target, the other will function as the test assistant.
1. OpenOCD and [arm-none-eabi-gdb] installed on your workstation. The latest official release of OpenOCD won't do, unfortunately. Use a recent version from Git, or the [xPack binaries].

In addition, you need to update some configuration, to reflect the realities on your system:

1. Update `test-target/dap-serial.cfg` and `test-assistant/dap-serial.cfg`, as described in those files. Otherwise OpenOCD/GDB won't be able to distinguish the two LPC845-BRK boards, and might try to upload both firmwares to the same device.
1. Update `test-suite/test-stand.toml` to make sure the test suite has the correct paths to the serial device files. Otherwise, it won't be able to communicate with the firmwares.

Once you have all of this set up, you can download the test target firmware like this:

```
cd test-target
cargo run
```

And the test assistant firmware like this:

```
cd test-assistant
cargo run
```

Once the firmware is running on the device, you can execute the test suite:

```
cd test-suite
cargo test
```

You should see a list of successfully executed test cases.

[LPC845-BRK]: https://www.nxp.com/products/processors-and-microcontrollers/arm-microcontrollers/general-purpose-mcus/lpc800-cortex-m0-plus-/lpc845-breakout-board-for-lpc84x-family-mcus:LPC845-BRK
[xPack binaries]: https://github.com/xpack-dev-tools/openocd-xpack/releases/
[arm-none-eabi-gdb]: https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm/downloads


## License

Code in this repository, unless specifically noted otherwise, is available under the terms under the [0BSD License]. This essentially means you can do what you want with it, without any restrictions.

See [LICENSE.md] for the full license text.

[0BSD License]: https://opensource.org/licenses/0BSD
[LICENSE.md]: LICENSE.md

**Created by [Braun Embedded](https://braun-embedded.com/)** <br />
**Initial development sponsored by [Georg Fischer Signet](http://www.gfsignet.com/)**
