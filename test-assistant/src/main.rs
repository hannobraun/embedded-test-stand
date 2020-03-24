//! Test assistant firmware
//!
//! Used to assist the test suite in interfacing with the test target. Needs to
//! be downloaded to an LPC845-BRK board before the test cases can be run.


#![no_main]
#![no_std]


// Includes a panic handler that outputs panic messages via semihosting. These
// should show up in OpenOCD.
extern crate panic_semihosting;


use heapless::{
    consts::U16,
    spsc::{
        Consumer,
        Producer,
        Queue,
    },
};
use lpc8xx_hal::{
    Peripherals,
    cortex_m::{
        asm,
        interrupt,
    },
    init_state::Enabled,
    pac::{
        USART0,
        USART1,
    },
    pinint::{
        self,
        PININT0,
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
    AssistantToHost,
    HostToAssistant,
};


#[rtfm::app(device = lpc8xx_hal::pac)]
const APP: () = {
    struct Resources {
        host_rx_int:  RxInt<'static, USART0>,
        host_rx_idle: RxIdle<'static>,
        host_tx:      Tx<USART0>,

        target_rx_int:  RxInt<'static, USART1>,
        target_rx_idle: RxIdle<'static>,
        target_tx:      Tx<USART1>,

        pin_prod: Producer<'static, bool, U16>,
        pin_cons: Consumer<'static, bool, U16>,
        pin_int:  pinint::Interrupt<PININT0, PIO1_0, Enabled>,
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        // Normally, access to a `static mut` would be unsafe, but we know that
        // this method is only called once, which means we have exclusive access
        // here. RTFM knows this too, and by putting these statics right here,
        // at the beginning of the method, we're opting into some RTFM magic
        // that gives us safe access to them.
        static mut HOST:   Usart = Usart::new();
        static mut TARGET: Usart = Usart::new();

        static mut PIN_QUEUE: Queue<bool, U16> =
            Queue(heapless::i::Queue::new());

        // Get access to the device's peripherals. This can't panic, since this
        // is the only place in this program where we call this method.
        let p = Peripherals::take().unwrap_or_else(|| unreachable!());

        let mut syscon = p.SYSCON.split();
        let     swm    = p.SWM.split();
        let     gpio   = p.GPIO.enable(&mut syscon.handle);
        let     pinint = p.PININT.enable(&mut syscon.handle);

        let mut swm_handle = swm.handle.enable(&mut syscon.handle);

        // Configure interrupt for pin connected to assistant's LED pin
        let _pin = p.pins.pio1_0.into_input_pin(gpio.tokens.pio1_0);
        let mut pin_int = pinint
            .interrupts
            .pinint0
            .select::<PIO1_0>(&mut syscon.handle);
        pin_int.enable_rising_edge();
        pin_int.enable_falling_edge();

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

        // Use USART1 to communicate with the test target
        let mut target = p.USART1.enable(
            &clock_config,
            &mut syscon.handle,
            u1_rxd,
            u1_txd,
        );
        target.enable_rxrdy();

        let (host_rx_int,   host_rx_idle,   host_tx)   = HOST.init(host);
        let (target_rx_int, target_rx_idle, target_tx) = TARGET.init(target);

        let (pin_prod, pin_cons) = PIN_QUEUE.split();

        init::LateResources {
            host_rx_int,
            host_rx_idle,
            host_tx,

            target_rx_int,
            target_rx_idle,
            target_tx,

            pin_prod,
            pin_cons,
            pin_int,
        }
    }

    #[idle(
        resources = [
            host_rx_idle,
            host_tx,
            target_rx_idle,
            target_tx,
            pin_cons,
        ]
    )]
    fn idle(cx: idle::Context) -> ! {
        let host_rx   = cx.resources.host_rx_idle;
        let host_tx   = cx.resources.host_tx;
        let target_rx = cx.resources.target_rx_idle;
        let target_tx = cx.resources.target_tx;
        let pin_queue = cx.resources.pin_cons;

        let mut buf = [0; 256];

        loop {
            target_rx
                .process_raw(|data| {
                    host_tx.send_message(
                        &AssistantToHost::UsartReceive(data),
                        &mut buf,
                    )
                })
                .expect("Error processing USART data");

            host_rx
                .process_message(|message| {
                    match message {
                        HostToAssistant::SendUsart(data) => {
                            target_tx.send_raw(data)
                        }
                    }
                })
                .expect("Error processing host request");
            host_rx.clear_buf();

            while let Some(level) = pin_queue.dequeue() {
                match level {
                    true => {
                        host_tx
                            .send_message(
                                &AssistantToHost::PinIsHigh,
                                &mut buf,
                            )
                            .unwrap();
                    }
                    false => {
                        host_tx
                            .send_message(
                                &AssistantToHost::PinIsLow,
                                &mut buf,
                            )
                            .unwrap();
                    }
                }
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
                let should_sleep =
                    !host_rx.can_process()
                    && !target_rx.can_process()
                    && !pin_queue.ready();

                if should_sleep {
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

    #[task(binds = USART1, resources = [target_rx_int])]
    fn usart1(cx: usart1::Context) {
        cx.resources.target_rx_int.receive()
            .expect("Error receiving from USART1");
    }

    #[task(binds = PIN_INT0, resources = [pin_prod, pin_int])]
    fn pinint0(context: pinint0::Context) {
        let queue = context.resources.pin_prod;
        let int   = context.resources.pin_int;

        if int.clear_rising_edge_flag() {
            queue.enqueue(true).unwrap();
        }
        if int.clear_falling_edge_flag() {
            queue.enqueue(false).unwrap();
        }
    }
};
