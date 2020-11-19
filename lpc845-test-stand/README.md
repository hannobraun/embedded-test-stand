# LPC845 Test Stand

## About

Test Stand for some of the peripheral APIs in [LPC8xx HAL].


## Structure

- `messages`: The messages used to communicate between test suite and target firmware.
- `test-target`: The firmware running on the hardware under test.
- `test-assistant`: The firmware running on the test assistant. This might be transformed into a part of the generic test stand infrastructure. See issue [#86](https://github.com/braun-embedded/lpc845-test-stand/issues/86).
- `test-suite`: The test suite itself, plus some suite-specific convenience wrappers around APIs in `host-lib`.


## Running the test suite

This repository contains an example test suite that sends commands to the device under test and verifies its response. This section describes what prerequisites you need for running the test suite, and how to do so.

### Hardware setup

You need two [LPC845-BRK] boards. One will be the test target, the other will serve as the test assistant.

Connect both boards to the host computer via their USB ports. This is required both to download the firmware and to communicate with it during the test.

In addition, you need to connect the following pins of the target and the assistant:

| Target Pin | Assistant Pin | Note                                       |
| ---------- | ------------- | ------------------------------------------ |
|          1 |             1 | SPI: SCK                                   |
|          2 |             2 | SPI: MOSI                                  |
|          3 |             3 | SPI: MISO                                  |
|          4 |             4 | SPI: SSEL                                  |
|         12 |            13 | USART: Target RX, Assistant TX             |
|         13 |            12 | USART: Target TX, Assistant RX             |
|         14 |            15 | USART: Target RX (DMA), Assistant TX       |
|         18 |            18 | USART: RTS                                 |
|         19 |            19 | USART: CTS                                 |
|         20 |            20 | GND (common ground for I2C)                |
|         23 |            23 | I2C: SCL (also connect pull-up resistor)   |
|         24 |            24 | I2C: SDA (also connect pull-up resistor)   |
|         26 |            27 | USART (sync): Target RX, Assistant TX      |
|         27 |            26 | USART (sync): Target TX, Assistant RX      |
|         28 |            28 | USART (sync): SCLK                         |
|         29 |            29 | GPIO: Target In, Assistant Out (red LED)   |
|         30 |            30 | Timer interrupt signal (blue LED)          |
|         31 |            31 | GPIO: Target Out, Assistant In (green LED) |

10 kOhm resistors are confirmed to work for the I2C pull-ups.

### Software setup

Besides a Rust toolchain, you need `cargo-embed` to download the firmware:
```
cargo install cargo-embed --version 0.8.0
```

#### Serial Number

Since the setup uses two identical LPC845-BRK boards, `cargo-embed` needs some way to distinguish between them. For this reason, the configuration files (`test-target/Embed.toml` and `test-assistant/Embed.toml`) specify a serial number in the `probe_selector` configuration.

Either update the serial number there, or override `probe_selector` in an `Embed.local.toml`.

On **Linux**, you can figure out the serial number of your device like this:
```console
$ udevadm info /path/to/device`
```
and watch out for `ID_SERIAL_SHORT` in the output.

On **macOS**, run
```console
$ system_profiler SPUSBDataType
```
and look for an entry called `LPC11U3x CMSIS-DAP v1.0.7` or similar. Copy the contents of the `Serial Number:` field into your `Embed.toml`s.

#### Serial Device Paths

You'll also need to check whether the serial device paths specified in `test-stand.toml` are correct.

To check the right serial device paths on **macOS**, run
```console
$ ls /dev/ | grep tty.usb
```
and look for tty devices called `tty.usbmodem`. The `usbmodem` devices you're looking for will likely have the lpc845's serial number in their name. Edit the serial device paths specified in `test-stand.toml` to match the `target` and `assistant`s modem path respectively.

### Running

Once you have all of this set up, you can download the test target firmware like this:

```
cd test-target
cargo embed
```

And the test assistant firmware like this:

```
cd test-assistant
cargo embed
```

Once the firmware is running on the device, you can execute the test suite:

```
cd test-suite
cargo test
```

You should see a list of successfully executed test cases.

### Troubleshooting

I make sure that the test suite runs reliably on my machine before merging any changes. While it is always possible that I missed a bug (please open an issue, if you find one!), the most common source of problems is the set-up.

Here are some tips to help you find problems:

- Make sure that the serial device paths you specified in `test-stand.toml` are correct. Please note that the path that is assigned to the target's or assistant's serial device can depend on the order in which they are connected to the host PC.
- Make sure that the correct version of the firmware is running on the devices. If you recently checked out another commit (maybe switched to another branch?), make sure your firmwares match your test suite by re-uploading them.
- Make sure the target and assistant are connected as documented above, and that no connections are loose or faulty.
- Make sure that both firmwares are in a valid state. They should be in a valid state after reset, and a successful test run should also leave them in a valid state. But a failed test run could render them unable to perform any more tests successfully.
- Make sure the serial device is in a valid state. A failed test run can leave unprocessed bytes in the serial device's read buffer. These bytes will be read on the next test run, confusing the test suite. You should be able to fix this problem by physically disconnecting and reconnecting the USB connections (make sure to reconnect them in the right order, so they match the configuration in `test-stand.toml`).
- Make sure there are no inactive logic analyzers connected. A logic analyzer that was connected to the I2C lines, but wasn't connected to the host PC via USB, has been known to interfere with I2C operations.

These are just some suggestions. Please feel free to add more, if you experience any more problems. There are currently open issues ([#6], [#46]) that would help make the whole setup more robust.

[LPC8xx HAL]: https://github.com/lpc-rs/lpc8xx-hal
[LPC845-BRK]: https://www.nxp.com/products/processors-and-microcontrollers/arm-microcontrollers/general-purpose-mcus/lpc800-cortex-m0-plus-/lpc845-breakout-board-for-lpc84x-family-mcus:LPC845-BRK
[xPack binaries]: https://github.com/xpack-dev-tools/openocd-xpack/releases/
[arm-none-eabi-gdb]: https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm/downloads
[#6]: https://github.com/braun-embedded/lpc845-test-stand/issues/6
[#46]: https://github.com/braun-embedded/lpc845-test-stand/issues/46
