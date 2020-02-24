# LPC845 Test Stand

## Introduction

A test stand for firmware applications. Allows you to control an application under test while it is running on the hardware, and verify that it behaves correctly.

While it is desirable to make this project available for all hardware eventually, it only supports NXP LPC845 microcontrollers right now, to make the initial development easier.


## Status

Implementation work has started, and a small (but growing) test suite covering LPC8xx HAL APIs is available. Work is being done to extend this test suite, and in doing so, make the infrastructure supporting it more useful and ready for real-world use.

No work is being done right now to support other microcontrollers than the LPC845. This would be a welcome addition though, so please get in touch by opening an issue, if you're interested.


## Structure

At the time of writing, there are 5 Cargo packages in this repository. It's a bit unfortunate that there are this many, but it doesn't make sense to reduce the number, because of the following factors:

1. There is firmware code targetting microcontrollers and code intended to be run on the host computer. Those have very different requirements regarding their dependencies, which makes having them in the same crate more trouble than it would be worth.
1. Right now, there are basically two projects in this repository: The test framework, and the test suite for LPC845 HAL. Eventually, both can live in separate places, but for now it makes sense to develop them together.

### Test Framework Crates

These are the crates that are independent of the LPC845 test suite. If you want to use this test stand for your own project, these are the crates you want to use:

- `firmware-lib`: Library for firmware running on the test target.
- `host-lib`: Library for test suites running on the host.

### LPC845 Test Suite

These crates belong to the LPC845 test suite. They are not directly applicable to other uses, except to serve as an example:

- `messages`: The messages used to communicate between test suite and target firmware.
- `test-firmware`: The firmware running on the hardware under test.
- `test-suite`: The test suite itself, plus some suite-specific convenience wrappers around APIs in `host-lib`.


## Running the Test Suite

This repository contains an example test suite that sends commands to the device under test and verifies its response.

Before you can run the test suite, you first need to download the firmware to an [LPC845-BRK]. This requires [arm-none-eabi-gdb] and OpenOCD (the latest official release won't do; use a recent version from Git, or the [xPack binaries]).

If you have those installed, you can download the test firmware like this:

```
cd test-firmware
cargo run
```

Once the firmware is running on the device, you can execute the test suite:

```
cd test-suite
cargo test
```

Depending on which system you're running this on, you might need to adapt the test stand configuration file (`test-suite/test-stand.toml`) to your needs.

You should see a list of successfully execute test cases.

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
