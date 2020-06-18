//! Firmware for the LPC845 HAL test suite
//!
//! Needs to be downloaded to the LPC845-BRK board before the test cases can be
//! run.


#![no_main]
#![no_std]


extern crate panic_rtt_target;


use core::marker::PhantomData;

use lpc8xx_hal::{
    prelude::*,
    Peripherals,
    cortex_m::{
        interrupt,
        peripheral::SYST,
    },
    gpio::{
        GpioPin,
        Level,
        direction::{
            Input,
            Output,
        },
    },
    i2c,
    init_state::Enabled,
    pac::{
        I2C0,
        USART0,
        USART1,
    },
    pinint::{
        self,
        PININT0,
    },
    pins::{
        PIO1_0,
        PIO1_1,
        PIO1_2,
    },
    syscon::{
        IOSC,
        frg,
    },
    usart,
};
use rtt_target::rprintln;

#[cfg(feature = "sleep")]
use lpc8xx_hal::cortex_m::asm;

use firmware_lib::usart::{
    RxIdle,
    RxInt,
    Tx,
    Usart,
};
use lpc845_messages::{
    HostToTarget,
    PinState,
    TargetToHost,
};


#[rtic::app(device = lpc8xx_hal::pac)]
const APP: () = {
    struct Resources {
        host_rx_int:  RxInt<'static, USART0>,
        host_rx_idle: RxIdle<'static>,
        host_tx:      Tx<USART0>,

        usart_rx_int:  RxInt<'static, USART1>,
        usart_rx_idle: RxIdle<'static>,
        usart_tx:      Tx<USART1>,

        green: GpioPin<PIO1_0, Output>,
        blue:  GpioPin<PIO1_1, Output>,
        red:   GpioPin<PIO1_2, Input>,

        red_int: pinint::Interrupt<PININT0, PIO1_2, Enabled>,

        systick: SYST,
        i2c:     i2c::Master<I2C0, Enabled<PhantomData<IOSC>>, Enabled>,
    }

    #[init]
    fn init(context: init::Context) -> init::LateResources {
        // Normally, access to a `static mut` would be unsafe, but we know that
        // this method is only called once, which means we have exclusive access
        // here. RTFM knows this too, and by putting these statics right here,
        // at the beginning of the method, we're opting into some RTFM magic
        // that gives us safe access to them.
        static mut HOST:  Usart = Usart::new();
        static mut USART: Usart = Usart::new();

        rtt_target::rtt_init_print!();
        rprintln!("Starting target.");

        // Get access to the device's peripherals. This can't panic, since this
        // is the only place in this program where we call this method.
        let p = Peripherals::take().unwrap_or_else(|| unreachable!());

        let systick = context.core.SYST;

        let mut syscon = p.SYSCON.split();
        let     swm    = p.SWM.split();
        let     gpio   = p.GPIO.enable(&mut syscon.handle);
        let     pinint = p.PININT.enable(&mut syscon.handle);

        let mut swm_handle = swm.handle.enable(&mut syscon.handle);

        // Configure GPIO pins
        let green = p.pins.pio1_0
            .into_output_pin(gpio.tokens.pio1_0, Level::High);
        let blue = p.pins.pio1_1
            .into_output_pin(gpio.tokens.pio1_1, Level::High);
        let red = p.pins.pio1_2
            .into_input_pin(gpio.tokens.pio1_2);

        // Set up interrupt for input pin
        let mut red_int = pinint
            .interrupts
            .pinint0
            .select::<PIO1_2>(&mut syscon.handle);
        red_int.enable_rising_edge();
        red_int.enable_falling_edge();

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

        let (i2c0_sda, _) = swm
            .fixed_functions
            .i2c0_sda
            .assign(p.pins.pio0_11.into_swm_pin(), &mut swm_handle);
        let (i2c0_scl, _) = swm
            .fixed_functions
            .i2c0_scl
            .assign(p.pins.pio0_10.into_swm_pin(), &mut swm_handle);

        let i2c = p.I2C0
            .enable(
                &syscon.iosc,
                i2c0_scl,
                i2c0_sda,
                &mut syscon.handle,
            )
            .enable_master_mode(
                &i2c::Clock::new_400khz(),
            );

        init::LateResources {
            host_rx_int,
            host_rx_idle,
            host_tx,

            usart_rx_int,
            usart_rx_idle,
            usart_tx,

            green,
            blue,
            red,

            red_int,

            systick,
            i2c: i2c.master,
        }
    }

    #[idle(resources = [
        host_rx_idle, host_tx,
        usart_rx_idle, usart_tx,
        green,
        red,
        systick,
        i2c,
    ])]
    fn idle(cx: idle::Context) -> ! {
        let usart_rx = cx.resources.usart_rx_idle;
        let usart_tx = cx.resources.usart_tx;
        let host_rx  = cx.resources.host_rx_idle;
        let host_tx  = cx.resources.host_tx;
        let green    = cx.resources.green;
        let red      = cx.resources.red;
        let systick  = cx.resources.systick;
        let i2c      = cx.resources.i2c;

        let mut buf = [0; 256];

        let mut input_was_high = red.is_high();

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
                        HostToTarget::SetPin(PinState::High) => {
                            Ok(green.set_high())
                        }
                        HostToTarget::SetPin(PinState::Low) => {
                            Ok(green.set_low())
                        }
                        HostToTarget::StartTimerInterrupt { period_ms } => {
                            // By default (and we haven't changed that setting)
                            // the SysTick timer runs at half the system
                            // frequency. The system frequency runs at 12 MHz by
                            // default (again, we haven't changed it), meaning
                            // the SysTick timer runs at 6 MHz.
                            //
                            // At 6 MHz, 1 ms are 6000 timer ticks.
                            let reload = period_ms * 6000;
                            systick.set_reload(reload);

                            systick.clear_current();
                            systick.enable_interrupt();
                            systick.enable_counter();

                            Ok(())
                        }
                        HostToTarget::StopTimerInterrupt => {
                            systick.disable_interrupt();
                            systick.disable_counter();

                            Ok(())
                        }
                        HostToTarget::StartI2cTransaction { address, data } => {
                            rprintln!("I2C: Write");
                            i2c.write(address, &[data])
                                .unwrap();

                            rprintln!("I2C: Read");
                            let mut rx_buf = [0u8; 1];
                            i2c.read(address, &mut rx_buf)
                                .unwrap();

                            rprintln!("I2C: Done");

                            host_tx
                                .send_message(
                                    &TargetToHost::I2cReply(rx_buf[0]),
                                    &mut buf,
                                )
                                .unwrap();

                            Ok(())
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
                let input_is_high = red.is_high();
                if input_is_high != input_was_high {
                    let level = match input_is_high {
                        true  => PinState::High,
                        false => PinState::Low,
                    };

                    host_tx
                        .send_message(
                            &TargetToHost::PinLevelChanged { level },
                            &mut buf,
                        )
                        .unwrap();

                    input_was_high = input_is_high;
                }

                if !host_rx.can_process() && !usart_rx.can_process() {
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

    #[task(binds = USART1, resources = [usart_rx_int])]
    fn usart1(cx: usart1::Context) {
        cx.resources.usart_rx_int.receive()
            .expect("Error receiving from USART1");
    }

    #[task(binds = SysTick, resources = [blue])]
    fn syst(cx: syst::Context) {
        cx.resources.blue.toggle();
    }

    #[task(binds = PIN_INT0, resources = [red_int])]
    fn pinint0(context: pinint0::Context) {
        let red_int = context.resources.red_int;

        red_int.clear_rising_edge_flag();
        red_int.clear_falling_edge_flag();
    }
};
