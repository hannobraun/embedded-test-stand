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
    ArrayLength,
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

use lpc845_test_lib::{
    Event,
    Receiver,
    Request,
};


#[rtfm::app(device = lpc8xx_hal::pac)]
const APP: () = {
    struct Resources {
        host_rx: usart::Rx<USART0>,
        host_tx: usart::Tx<USART0>,

        usart_rx: usart::Rx<USART1>,
        usart_tx: usart::Tx<USART1>,

        request_prod: spsc::Producer<'static, u8, U256>,
        request_cons: spsc::Consumer<'static, u8, U256>,

        usart_prod: spsc::Producer<'static, u8, U256>,
        usart_cons: spsc::Consumer<'static, u8, U256>,
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        // Normally, access to a `static mut` would be unsafe, but we know that
        // this method is only called once, which means we have exclusive access
        // here. RTFM knows this too, and by putting these statics right here,
        // at the beginning of the method, we're opting into some RTFM magic
        // that gives us safe access to them.
        static mut HOST_QUEUE: spsc::Queue<u8, U256> =
            spsc::Queue(heapless::i::Queue::new());
        static mut USART_QUEUE: spsc::Queue<u8, U256> =
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
        let mut usart = p.USART1.enable(
            &clock_config,
            &mut syscon.handle,
            u1_rxd,
            u1_txd,
        );
        usart.enable_rxrdy();

        let (request_prod, request_cons) = HOST_QUEUE.split();
        let (usart_prod,   usart_cons)   = USART_QUEUE.split();

        init::LateResources {
            host_rx:  host.rx,
            host_tx:  host.tx,
            usart_rx: usart.rx,
            usart_tx: usart.tx,
            request_prod,
            request_cons,
            usart_prod,
            usart_cons,
        }
    }

    #[idle(resources = [host_tx, usart_tx, request_cons, usart_cons])]
    fn idle(cx: idle::Context) -> ! {
        let host        = cx.resources.host_tx;
        let usart       = cx.resources.usart_tx;
        let usart_queue = cx.resources.usart_cons;

        let mut request_receiver = Receiver::new(
            cx.resources.request_cons,
        );

        let mut usart_buf = [0; 256];
        let mut i         = 0;

        let mut serialize_buf = [0; 256];

        loop {
            while let Some(b) = usart_queue.dequeue() {
                usart_buf[i] = b;
                i += 1;
            }

            if i > 0 {
                Event::UsartReceive(&usart_buf[0..i])
                    .send(host, &mut serialize_buf)
                    .expect("Failed to send `UsartReceive` event");
                i = 0;
            }

            if let Some(request) = request_receiver.receive() {
                // Receive a request from the test suite and do whatever it
                // tells us.
                match request {
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

                request_receiver.reset();
            }

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
                if !request_receiver.can_receive() {
                    asm::wfi();
                }
            });
        }
    }

    #[task(binds = USART0, resources = [host_rx, request_prod])]
    fn receive_from_host(cx: receive_from_host::Context) {
        receive(cx.resources.host_rx, cx.resources.request_prod);
    }

    #[task(binds = USART1, resources = [usart_rx, usart_prod])]
    fn receive_from_usart(cx: receive_from_usart::Context) {
        receive(cx.resources.usart_rx, cx.resources.usart_prod);
    }
};


fn receive<USART, Capacity>(
    usart: &mut usart::Rx<USART>,
    queue: &mut spsc::Producer<u8, Capacity>,
)
    where
        USART:    usart::Instance,
        Capacity: ArrayLength<u8>,
{
    // We're ignoring all errors here, as there's nothing we can do about them
    // anyway. They will show up on the host as test failures.
    while let Ok(b) = usart.read() {
        let _ = queue.enqueue(b);
    }
}
