//! Firmware for the LPC845 HAL test suite
//!
//! Needs to be downloaded to the LPC845-BRK board before the test cases can be
//! run.


#![no_main]
#![no_std]


// Includes a panic handler that outputs panic messages via semihosting. These
// should show up in OpenOCD.
extern crate panic_semihosting;


use lpc8xx_hal::{
    prelude::*,
    Peripherals,
    cortex_m::{
        asm,
        interrupt,
    },
    gpio::{
        GpioPin,
        Level,
        direction::Output,
    },
    pac::{
        USART0,
        USART1,
    },
    pins::PIO1_0,
    syscon::frg,
    usart,
};

use firmware_lib::usart::{
    RxIdle,
    RxInt,
    Tx,
    Usart,
};
use lpc845_messages::{
    HostToTarget,
    TargetToHost,
};


#[rtfm::app(device = lpc8xx_hal::pac)]
const APP: () = {
    struct Resources {
        host_rx_int:  RxInt<'static, USART0>,
        host_rx_idle: RxIdle<'static>,
        host_tx:      Tx<USART0>,

        usart_rx_int:  RxInt<'static, USART1>,
        usart_rx_idle: RxIdle<'static>,
        usart_tx:      Tx<USART1>,

        led: GpioPin<PIO1_0, Output>,
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        // Normally, access to a `static mut` would be unsafe, but we know that
        // this method is only called once, which means we have exclusive access
        // here. RTFM knows this too, and by putting these statics right here,
        // at the beginning of the method, we're opting into some RTFM magic
        // that gives us safe access to them.
        static mut HOST:  Usart = Usart::new();
        static mut USART: Usart = Usart::new();

        // Get access to the device's peripherals. This can't panic, since this
        // is the only place in this program where we call this method.
        let p = Peripherals::take().unwrap_or_else(|| unreachable!());

        let mut syscon = p.SYSCON.split();
        let     swm    = p.SWM.split();
        let     gpio   = p.GPIO.enable(&mut syscon.handle);

        let mut swm_handle = swm.handle.enable(&mut syscon.handle);

        // Configure green LED for output.
        let led = p.pins.pio1_0
            .into_output_pin(gpio.tokens.pio1_0, Level::High);

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
            usart::Clock::new(&syscon.frg0, 5, 16)
        };

        // Assign pins to USART0 for RX/TX functions. On the LPC845-BRK, those
        // are the pins connected to the programmer, and bridged to the host via
        // USB.
        //
        // Careful, the LCP845-BRK documentation uses the opposite designations
        // (i.e. from the perspective of the on-board programmer, not the
        // microcontroller).
        let (u0_rxd, _) = swm.movable_functions.u0_rxd.assign(
            p.pins.pio0_24.into_swm_pin(),
            &mut swm_handle,
        );
        let (u0_txd, _) = swm.movable_functions.u0_txd.assign(
            p.pins.pio0_25.into_swm_pin(),
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

        // Assign pins to USART1.
        let (u1_rxd, _) = swm.movable_functions.u1_rxd.assign(
            p.pins.pio0_26.into_swm_pin(),
            &mut swm_handle,
        );
        let (u1_txd, _) = swm.movable_functions.u1_txd.assign(
            p.pins.pio0_27.into_swm_pin(),
            &mut swm_handle,
        );

        // Use USART1 as the test subject.
        let mut usart = p.USART1.enable(
            &clock_config,
            &mut syscon.handle,
            u1_rxd,
            u1_txd,
        );
        usart.enable_rxrdy();

        let (host_rx_int,  host_rx_idle,  host_tx)  = HOST.init(host);
        let (usart_rx_int, usart_rx_idle, usart_tx) = USART.init(usart);

        init::LateResources {
            host_rx_int,
            host_rx_idle,
            host_tx,

            usart_rx_int,
            usart_rx_idle,
            usart_tx,

            led,
        }
    }

    #[idle(resources = [host_rx_idle, host_tx, usart_rx_idle, usart_tx, led])]
    fn idle(cx: idle::Context) -> ! {
        let usart_rx = cx.resources.usart_rx_idle;
        let usart_tx = cx.resources.usart_tx;
        let host_rx  = cx.resources.host_rx_idle;
        let host_tx  = cx.resources.host_tx;
        let led      = cx.resources.led;

        let mut buf = [0; 256];

        loop {
            usart_rx
                .process_raw(|data| {
                    host_tx.send_message(
                        &TargetToHost::UsartReceive(data),
                        &mut buf,
                    )
                })
                .expect("Error processing USART data");

            host_rx
                .process_message(|message| {
                    match message {
                        HostToTarget::SendUsart(data) => {
                            usart_tx.send_raw(data)
                        }
                        HostToTarget::SetPinHigh => {
                            led.set_high()
                        }
                        HostToTarget::SetPinLow => {
                            led.set_low()
                        }
                    }
                })
                .expect("Error processing host request");
            host_rx.clear_buf();

            // We need this critical section to protect against a race
            // conditions with the interrupt handlers. Otherwise, the following
            // sequence of events could occur:
            // 1. We check the queues here, they're empty.
            // 2. New data is received, an interrupt handler adds it to a queue.
            // 3. The interrupt handler is done, we're back here and going to
            //    sleep.
            //
            // This might not be observable, if something else happens to wake
            // us up before the test suite times out. But it could also lead to
            // spurious test failures.
            interrupt::free(|_| {
                if !host_rx.can_process() && !usart_rx.can_process() {
                    asm::wfi();
                }
            });
        }
    }

    #[task(binds = USART0, resources = [host_rx_int])]
    fn usart0(cx: usart0::Context) {
        cx.resources.host_rx_int.receive()
            .expect("Error receiving from USART0");
    }

    #[task(binds = USART1, resources = [usart_rx_int])]
    fn usart1(cx: usart1::Context) {
        cx.resources.usart_rx_int.receive()
            .expect("Error receiving from USART1");
    }
};
