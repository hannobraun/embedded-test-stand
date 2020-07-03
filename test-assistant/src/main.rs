//! Test assistant firmware
//!
//! Used to assist the test suite in interfacing with the test target. Needs to
//! be downloaded to an LPC845-BRK board before the test cases can be run.


#![no_main]
#![no_std]


extern crate panic_rtt_target;


use lpc8xx_hal::{
    Peripherals,
    cortex_m::interrupt,
    gpio::{
        self,
        GpioPin,
        direction::Output,
    },
    mrt::{
        MRT0,
        MRT1,
    },
    pac::{
        USART0,
        USART1,
    },
    pinint::{
        PININT0,
        PININT1,
    },
    pins::{
        PIO1_0,
        PIO1_1,
        PIO1_2,
    },
    syscon::frg,
    usart,
};
use rtt_target::rprintln;

#[cfg(feature = "sleep")]
use lpc8xx_hal::cortex_m::asm;

use firmware_lib::{
    pin_interrupt::{
        self,
        PinInterrupt,
    },
    usart::{
        RxIdle,
        RxInt,
        Tx,
        Usart,
    },
};
use lpc845_messages::{
    AssistantToHost,
    HostToAssistant,
    Pin,
    PinState,
};


#[rtic::app(device = lpc8xx_hal::pac)]
const APP: () = {
    struct Resources {
        host_rx_int:  RxInt<'static, USART0>,
        host_rx_idle: RxIdle<'static>,
        host_tx:      Tx<USART0>,

        target_rx_int:  RxInt<'static, USART1>,
        target_rx_idle: RxIdle<'static>,
        target_tx:      Tx<USART1>,

        green_int:  pin_interrupt::Int<'static, PININT0, PIO1_0, MRT0>,
        green_idle: pin_interrupt::Idle<'static>,

        blue_int:  pin_interrupt::Int<'static, PININT1, PIO1_1, MRT1>,
        blue_idle: pin_interrupt::Idle<'static>,

        red: GpioPin<PIO1_2, Output>,
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

        static mut GREEN: PinInterrupt = PinInterrupt::new();
        static mut BLUE:  PinInterrupt = PinInterrupt::new();

        rtt_target::rtt_init_print!();
        rprintln!("Starting assistant.");

        // Get access to the device's peripherals. This can't panic, since this
        // is the only place in this program where we call this method.
        let p = Peripherals::take().unwrap_or_else(|| unreachable!());

        let mut syscon = p.SYSCON.split();
        let     swm    = p.SWM.split();
        let     gpio   = p.GPIO.enable(&mut syscon.handle);
        let     pinint = p.PININT.enable(&mut syscon.handle);
        let     timers = p.MRT0.split(&mut syscon.handle);

        let mut swm_handle = swm.handle.enable(&mut syscon.handle);

        // Configure interrupt for pin connected target's GPIO pin
        let _green = p.pins.pio1_0.into_input_pin(gpio.tokens.pio1_0);
        let mut green_int = pinint
            .interrupts
            .pinint0
            .select::<PIO1_0>(&mut syscon.handle);
        green_int.enable_rising_edge();
        green_int.enable_falling_edge();

        // Configure interrupt for pin connected to target's timer interrupt pin
        let _blue = p.pins.pio1_1.into_input_pin(gpio.tokens.pio1_1);
        let mut blue_int = pinint
            .interrupts
            .pinint1
            .select::<PIO1_1>(&mut syscon.handle);
        blue_int.enable_rising_edge();
        blue_int.enable_falling_edge();

        // Configure pin connected to target's input pin
        let red = p.pins.pio1_2.into_output_pin(
            gpio.tokens.pio1_2,
            gpio::Level::High,
        );

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

        let (green_int, green_idle) = GREEN.init(green_int, timers.mrt0);
        let (blue_int,  blue_idle)  = BLUE.init(blue_int, timers.mrt1);

        init::LateResources {
            host_rx_int,
            host_rx_idle,
            host_tx,

            target_rx_int,
            target_rx_idle,
            target_tx,

            green_int,
            green_idle,

            blue_int,
            blue_idle,

            red,
        }
    }

    #[idle(
        resources = [
            host_rx_idle,
            host_tx,
            target_rx_idle,
            target_tx,
            green_idle,
            blue_idle,
            red,
        ]
    )]
    fn idle(cx: idle::Context) -> ! {
        let host_rx   = cx.resources.host_rx_idle;
        let host_tx   = cx.resources.host_tx;
        let target_rx = cx.resources.target_rx_idle;
        let target_tx = cx.resources.target_tx;
        let green     = cx.resources.green_idle;
        let blue      = cx.resources.blue_idle;
        let red       = cx.resources.red;

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
                        HostToAssistant::SetPin(level) => {
                            match level {
                                PinState::High => {
                                    red.set_high();
                                }
                                PinState::Low => {
                                    red.set_low();
                                }
                            }
                            Ok(())
                        }
                    }
                })
                .expect("Error processing host request");
            host_rx.clear_buf();

            handle_timer_interrupts(green, Pin::Green, host_tx, &mut buf);
            handle_timer_interrupts(blue,  Pin::Blue,  host_tx, &mut buf);

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
                    && !green.is_ready();

                if should_sleep {
                    // On LPC84x MCUs, debug mode is not supported when
                    // sleeping. This interferes with RTT communication. Only
                    // sleep, if the user enables this through a compile-time
                    // flag.
                    #[cfg(feature = "sleep")]
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

    #[task(binds = PIN_INT0, resources = [green_int])]
    fn pinint0(context: pinint0::Context) {
        context.resources.green_int.handle_interrupt();
    }

    #[task(binds = PIN_INT1, resources = [blue_int])]
    fn pinint1(context: pinint1::Context) {
        context.resources.blue_int.handle_interrupt();
    }
};


fn handle_timer_interrupts<U>(
    int:     &mut pin_interrupt::Idle,
    pin:     Pin,
    host_tx: &mut Tx<U>,
    buf:     &mut [u8],
)
    where
        U: usart::Instance,
{
    while let Some(event) = int.next() {
        match event {
            pin_interrupt::Event { level, period } => {
                let level = match level {
                    gpio::Level::High => PinState::High,
                    gpio::Level::Low  => PinState::Low,
                };

                let period_ms = period.map(|value| value / 12_000);
                host_tx
                    .send_message(
                        &AssistantToHost::PinLevelChanged {
                            pin,
                            level,
                            period_ms,
                        },
                        buf,
                    )
                    .unwrap();
            }
        }
    }
}
