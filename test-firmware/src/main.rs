//! Firmware for the LPC845 HAL test suite
//!
//! Needs to be downloaded to the LPC845-BRK board before the test cases can be
//! run.


#![no_main]
#![no_std]


// Includes a panic handler that outputs panic messages via semihosting. These
// should show up in OpenOCD.
extern crate panic_semihosting;


use cortex_m_semihosting::hprintln;
use lpc8xx_hal::{
    prelude::*,
    Peripherals,
    USART,
    cortex_m_rt::entry,
    nb::block,
    syscon::{
        clocksource::PeripheralClockConfig,
        frg,
    },
    usart,
};
use void::ResultVoidExt;


#[entry]
fn main() -> ! {
    // Get access to the device's peripherals. This can't panic, since this is
    // the only place in this program where we call this method.
    let p = Peripherals::take().unwrap_or_else(|| unreachable!());

    let mut syscon = p.SYSCON.split();
    let     swm    = p.SWM.split();

    let mut swm_handle = swm.handle.enable(&mut syscon.handle);

    // Configure the clock for the USART, using the Fractional Rate Generator
    // (FRG) and the USART's own baud rate divider value (BRG). See user manual,
    // section 17.7.1.
    //
    // This assumes a system clock of 12 MHz (which is the default and, as of
    // this writing, has not been changed in this program). The resulting rate
    // is roughly 115200 baud.
    let clock_config = {
        syscon.frg0.select_clock(frg::Clock::FRO);
        syscon.frg0.set_mult(22);
        syscon.frg0.set_div(0xFF);
        PeripheralClockConfig::new(&syscon.frg0, 5)
    };

    // The pins used for USART RX/TX. On the LPC845-BRK, those are the pins
    // connected to the programmer, and bridged to the host via USB.
    //
    // Careful, the LCP845-BRK documentation uses the opposite designations
    // (i.e. from the perspective of the programmer, not the microcontroller).
    let rx_pin = swm.pins.pio0_24.into_swm_pin();
    let tx_pin = swm.pins.pio0_25.into_swm_pin();

    // Assign the USART functions to the pins.
    let (u0_rxd, _) = swm.movable_functions.u0_rxd
        .assign(rx_pin, &mut swm_handle);
    let (u0_txd, _) = swm.movable_functions.u0_txd
        .assign(tx_pin, &mut swm_handle);

    // Enable USART0
    let mut usart = p.USART0.enable(
        &clock_config,
        &mut syscon.handle,
        u0_rxd,
        u0_txd,
    );

    // Eventually, we'll execute commands from the test suite here, so it can
    // verify we're doing the right thing. For now, let's just echo everything
    // we receive, so the test suite can implement a basic test for USART.
    loop {
        if let Err(err) = echo(&mut usart) {
            // Nothing we can do really. Let's just send an error message to the
            // host via semihosting and carry on.
            let _ = hprintln!("Error echoing byte via USART: {:?}", err);
        }
    }
}

fn echo<I: usart::Peripheral>(usart: &mut USART<I>)
    -> Result<(), usart::Error>
{
    let b = block!(usart.rx().read())?;
    block!(usart.tx().write(b))
        .void_unwrap();
    Ok(())
}
