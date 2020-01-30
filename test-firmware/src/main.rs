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
    cortex_m_rt::entry,
    syscon::{
        clocksource::UsartClock,
        frg,
    },
};
use void::ResultVoidExt;

use lpc845_test_lib::Request;


#[entry]
fn main() -> ! {
    // Get access to the device's peripherals. This can't panic, since this is
    // the only place in this program where we call this method.
    let p = Peripherals::take().unwrap_or_else(|| unreachable!());

    let mut syscon = p.SYSCON.split();
    let     swm    = p.SWM.split();

    let mut swm_handle = swm.handle.enable(&mut syscon.handle);

    // Configure the clock for USART0, using the Fractional Rate Generator (FRG)
    // and the USART's own baud rate divider value (BRG). See user manual,
    // section 17.7.1.
    //
    // This assumes a system clock of 12 MHz (which is the default and, as of
    // this writing, has not been changed in this program). The resulting rate
    // is roughly 115200 baud.
    let clock_config = {
        syscon.frg0.select_clock(frg::Clock::FRO);
        syscon.frg0.set_mult(22);
        syscon.frg0.set_div(0xFF);
        UsartClock::new(&syscon.frg0, 5, 16)
    };

    // Assign pins to USART0 for RX/TX functions. On the LPC845-BRK, those are
    // the pins connected to the programmer, and bridged to the host via USB.
    //
    // Careful, the LCP845-BRK documentation uses the opposite designations
    // (i.e. from the perspective of the on-boardprogrammer, not the
    // microcontroller).
    let (u0_rxd, _) = swm.movable_functions.u0_rxd.assign(
        swm.pins.pio0_24.into_swm_pin(),
        &mut swm_handle,
    );
    let (u0_txd, _) = swm.movable_functions.u0_txd.assign(
        swm.pins.pio0_25.into_swm_pin(),
        &mut swm_handle,
    );

    // Use USART0 to communicate with the test suite
    let mut host = p.USART0.enable(
        &clock_config,
        &mut syscon.handle,
        u0_rxd,
        u0_txd,
    );

    // USART0 is already set up for 115200 baud, which is also fine for USART1.
    // Let's reuse the FRG0 configuration.
    //
    // Please note that as of this writing, a bug in LPC8xx HAL would allow us
    // to overwrite the FRG0 configuration here, which would totally mess up
    // USART0's baud rate. If you want to change the baud rate for USART1,
    // please use a different clock (like FRG1), or use the same clock and pass
    // different divider values to `UsartClock::new`.
    let clock_config = UsartClock::new(&syscon.frg0, 5, 16);

    // Assign pins to USART1.
    let (u1_rxd, _) = swm.movable_functions.u1_rxd.assign(
        swm.pins.pio0_26.into_swm_pin(),
        &mut swm_handle,
    );
    let (u1_txd, _) = swm.movable_functions.u1_txd.assign(
        swm.pins.pio0_27.into_swm_pin(),
        &mut swm_handle,
    );

    // Use USART1 as the test subject.
    let usart = p.USART1.enable(
        &clock_config,
        &mut syscon.handle,
        u1_rxd,
        u1_txd,
    );

    let mut buf = [0; 256];

    loop {
        // Receive a request from the test suite and do whatever it tells us.
        match Request::receive(&mut host, &mut buf) {
            Ok(Request::SendUsart(message)) => {
                usart.tx().bwrite_all(message)
                    .void_unwrap();
            }
            Err(err) => {
                // Nothing we can do really. Let's just send an error message to
                // the host via semihosting and carry on.
                let _ = hprintln!("Error echoing byte via USART: {:?}", err);
            }
        }
    }
}
