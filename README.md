# LPC845 Test Stand

## Introduction

A test stand for firmware applications. Allows you to control an application under test while it is running on the hardware, and verify that it behaves correctly.

The long-term vision is to turn this into a generally applicable product. For now, it only supports NXP LPC845 microcontrollers.


## Status

As of this writing, nothing but this README and a few ideas exist. Please check out the list of merged pull requests to see if any more progress has been made since then.


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
