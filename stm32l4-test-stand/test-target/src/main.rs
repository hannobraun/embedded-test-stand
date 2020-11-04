//! Firmware for the STM32L4 Test Stand


#![no_main]
#![no_std]


extern crate panic_rtt_target;


use cortex_m::asm;
use rtt_target::rprintln;


#[rtic::app(device = stm32l4xx_hal::pac)]
const APP: () = {
    #[init]
    fn init(_cx: init::Context) {
        rtt_target::rtt_init_print!();
        rprintln!("Starting target.");
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            asm::nop();
        }
    }
};
