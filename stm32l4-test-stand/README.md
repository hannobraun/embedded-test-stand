# STM32L4 Test Stand

## About

Test stand for some peripheral APIs of [stm32l4xx-hal].

This test stand is modelled on the [LPC845 Test Stand] and re-uses some parts of it. Please check out the README there for more documentation and troubleshooting tips.


## Status

This test stand is under development. Not much to see yet.


## Hardware setup

You need the following development boards:

- Target: [STM32L433 Nucleo](https://www.st.com/en/evaluation-tools/nucleo-l433rc-p.html)
- Assistant: [LPC845-BRK](https://www.nxp.com/products/processors-and-microcontrollers/arm-microcontrollers/general-purpose-mcus/lpc800-cortex-m0-plus-/lpc845-breakout-board-for-lpc84x-family-mcus:LPC845-BRK)

Connect both boards to the host computer via their USB ports. This is required both to download the firmware and to communicate with it during the test.

In addition, you need to connect the following pins of the target and the assistant:

| Target | Assistant | Note                           |
| ------ | --------- | ------------------------------ |
| CN7  1 |        12 | USART: Target TX, Assistant RX |
| CN7  9 |        13 | USART: Target RX, Assistant TX |


[stm32l4xx-hal]: https://github.com/stm32-rs/stm32l4xx-hal
[LPC845 Test Stand]: https://github.com/braun-embedded/embedded-test-stand/tree/master/lpc845-test-stand
