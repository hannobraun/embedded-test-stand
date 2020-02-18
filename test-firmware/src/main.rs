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
use heapless::{
    consts::U256,
    spsc,
};
use lpc8xx_hal::{
    prelude::*,
    Peripherals,
    cortex_m::{
        asm,
        interrupt,
    },
    pac::{
        USART0,
        USART1,
    },
    syscon::{
        clocksource::UsartClock,
        frg,
    },
    usart,
};
use void::ResultVoidExt;

use lpc845_test_lib::Request;


#[rtfm::app(device = lpc8xx_hal::pac)]
const APP: () = {
    struct Resources {
        host_rx:  usart::Rx<USART0>,
        usart_tx: usart::Tx<USART1>,

        request_prod: spsc::Producer<'static, u8, U256>,
        request_cons: spsc::Consumer<'static, u8, U256>,
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        // Normally, access to a `static mut` would be unsafe, but we know that
        // this method is only called once, which means we have exclusive access
        // here. RTFM knows this too, and by putting this static right here, at
        // the beginning of the method, we're opting into some RTFM magic that
        // gives us safe access to the static.
        static mut HOST_QUEUE: spsc::Queue<u8, U256> =
            spsc::Queue(heapless::i::Queue::new());

        // Get access to the device's peripherals. This can't panic, since this
        // is the only place in this program where we call this method.
        let p = Peripherals::take().unwrap_or_else(|| unreachable!());

        let mut syscon = p.SYSCON.split();
        let     swm    = p.SWM.split();

        let mut swm_handle = swm.handle.enable(&mut syscon.handle);

        // Configure the clock for USART0, using the Fractional Rate Generator
        // (FRG) and the USART's own baud rate divider value (BRG). See user
        // manual, section 17.7.1.
        //
        // This assumes a system clock of 12 MHz (which is the default and, as
        // of this writing, has not been changed in this program). The resulting
        // rate is roughly 115200 baud.
        let clock_config = {
            syscon.frg0.select_clock(frg::Clock::FRO);
            syscon.frg0.set_mult(22);
            syscon.frg0.set_div(0xFF);
            UsartClock::new(&syscon.frg0, 5, 16)
        };

        // Assign pins to USART0 for RX/TX functions. On the LPC845-BRK, those
        // are
        // the pins connected to the programmer, and bridged to the host via
        // USB.
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
        host.enable_rxrdy();

        // USART0 is already set up for 115200 baud, which is also fine for
        // USART1. Let's reuse the FRG0 configuration.
        //
        // Please note that as of this writing, a bug in LPC8xx HAL would allow
        // us to overwrite the FRG0 configuration here, which would totally mess
        // up USART0's baud rate. If you want to change the baud rate for
        // USART1, please use a different clock (like FRG1), or use the same
        // clock and pass different divider values to `UsartClock::new`.
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

        let (request_prod, request_cons) = HOST_QUEUE.split();

        init::LateResources {
            host_rx: host.rx,
            usart_tx: usart.tx,
            request_prod,
            request_cons,
        }
    }

    #[idle(resources = [usart_tx, request_cons])]
    fn idle(cx: idle::Context) -> ! {
        let usart = cx.resources.usart_tx;
        let queue = cx.resources.request_cons;

        let mut buf = [0; 256];
        let mut i   = 0;

        loop {
            let mut request_received = false;

            while let Some(b) = queue.dequeue() {
                buf[i] = b;
                i += 1;

                // Requests are COBS-encoded, so we know that `0` means we
                // received a full frame.
                if b == 0 {
                    request_received = true;
                    break;
                }
            }

            if !request_received {
                // We need this critical section to protect against a race
                // condition with the interrupt handler. Otherwise, the
                // following sequence of events could happen:
                // 1. We check the queue here, it's empty.
                // 2. A new request is received, the interrupt handler adds it
                //    to the queue.
                // 3. The interrupt handler is done, we're back here and going
                //    to sleep.
                //
                // This might not be observable, if something else happens to
                // wake us up before the test suite times out, but it could lead
                // to spurious test failures.
                interrupt::free(|_| {
                    if !queue.ready() {
                        asm::wfi();
                    }
                });

                continue;
            }

            // Receive a request from the test suite and do whatever it tells
            // us.
            match Request::deserialize(&mut buf) {
                Ok(Request::SendUsart(message)) => {
                    usart.bwrite_all(message)
                        .void_unwrap();
                }
                Err(err) => {
                    // Nothing we can do really. Let's just send an error
                    // message to the host via semihosting and carry on.
                    let _ = hprintln!(
                        "Error receiving host request: {:?}",
                        err,
                    );
                }
            }

            // Reset the buffer.
            i = 0;
        }
    }

    #[task(binds = USART0, resources = [host_rx, request_prod])]
    fn receive_from_host(cx: receive_from_host::Context) {
        let host  = cx.resources.host_rx;
        let queue = cx.resources.request_prod;

        // We're ignoring all errors here, as there's nothing we can do about
        // them anyway. They will show up on the host as test failures.
        while let Ok(b) = host.read() {
            let _ = queue.enqueue(b);
        }
    }
};
