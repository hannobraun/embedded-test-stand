//! Test assistant firmware
//!
//! Used to assist the test suite in interfacing with the test target. Needs to
//! be downloaded to an LPC845-BRK board before the test cases can be run.


#![no_main]
#![no_std]


extern crate panic_rtt_target;


use core::marker::PhantomData;

use heapless::{
    FnvIndexMap,
    consts::U8,
};
use lpc8xx_hal::{
    prelude::*,
    Peripherals,
    cortex_m::interrupt,
    gpio::{
        self,
        GpioPin,
        direction::Output,
        direction::Input,
    },
    i2c,
    init_state::Enabled,
    mrt::{
        MRT0,
        MRT1,
        MRT2,
        MRT3,
    },
    nb::{
        self,
        block,
    },
    pac::{
        I2C0,
        SPI0,
        USART0,
        USART1,
        USART2,
        USART3,
    },
    pinint::{
        PININT0,
        PININT1,
        PININT2,
        PININT3,
    },
    pins::{
        PIO0_8,
        PIO0_9,
        PIO0_20,
        PIO0_23,
        PIO1_0,
        PIO1_1,
        PIO1_2,
    },
    spi::{
        self,
        SPI,
    },
    syscon::{
        IOSC,
        frg,
    },
    usart::{
        self,
        state::{
            AsyncMode,
            SyncMode,
        },
    },
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
    InputPin,
    OutputPin,
    UsartMode,
    pin,
};


#[rtic::app(device = lpc8xx_hal::pac)]
const APP: () = {
    struct Resources {
        host_rx_int:  RxInt<'static, USART0, AsyncMode>,
        host_rx_idle: RxIdle<'static>,
        host_tx:      Tx<USART0, AsyncMode>,

        target_rx_int:   RxInt<'static, USART1, AsyncMode>,
        target_rx_idle:  RxIdle<'static>,
        target_tx:       Tx<USART1, AsyncMode>,
        target_tx_dma:   usart::Tx<
            USART2,
            usart::state::Enabled<u8, AsyncMode>,
            usart::state::NoThrottle,
        >,
        target_rts_int:  pin_interrupt::Int<'static, PININT2, PIO0_9, MRT2>,
        target_rts_idle: pin_interrupt::Idle<'static>,

        target_sync_rx_int:  RxInt<'static, USART3, SyncMode>,
        target_sync_rx_idle: RxIdle<'static>,
        target_sync_tx:      Tx<USART3, SyncMode>,

        green_int:  pin_interrupt::Int<'static, PININT0, PIO1_0, MRT0>,
        green_idle: pin_interrupt::Idle<'static>,

        blue_int:  pin_interrupt::Int<'static, PININT1, PIO1_1, MRT1>,
        blue_idle: pin_interrupt::Idle<'static>,

        pwm_int:  pin_interrupt::Int<'static, PININT3, PIO0_23, MRT3>,
        pwm_idle: pin_interrupt::Idle<'static>,

        pin_5: GpioPin<PIO0_20, Output>,
        cts: GpioPin<PIO0_8, Output>,
        red: GpioPin<PIO1_2, Output>,
        green: GpioPin<PIO1_0, Input>,

        i2c: i2c::Slave<I2C0, Enabled<PhantomData<IOSC>>, Enabled>,
        spi: SPI<SPI0, Enabled<spi::Slave>>,
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        // Normally, access to a `static mut` would be unsafe, but we know that
        // this method is only called once, which means we have exclusive access
        // here. RTFM knows this too, and by putting these statics right here,
        // at the beginning of the method, we're opting into some RTFM magic
        // that gives us safe access to them.
        static mut HOST:        Usart = Usart::new();
        static mut TARGET:      Usart = Usart::new();
        static mut TARGET_SYNC: Usart = Usart::new();

        static mut GREEN: PinInterrupt = PinInterrupt::new();
        static mut BLUE:  PinInterrupt = PinInterrupt::new();
        static mut RTS:   PinInterrupt = PinInterrupt::new();
        static mut PWM:   PinInterrupt = PinInterrupt::new();

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
        let green = p.pins.pio1_0.into_input_pin(gpio.tokens.pio1_0);
        let mut green_int = pinint
            .interrupts
            .pinint0
            .select(green.inner(), &mut syscon.handle);
        green_int.enable_rising_edge();
        green_int.enable_falling_edge();

        // Configure interrupt for pin connected to target's timer interrupt pin
        let blue = p.pins.pio1_1.into_input_pin(gpio.tokens.pio1_1);
        let mut blue_int = pinint
            .interrupts
            .pinint1
            .select(blue.inner(), &mut syscon.handle);
        blue_int.enable_rising_edge();
        blue_int.enable_falling_edge();

        // Configure interrupt for pin connected to target's PWM pin
        let pwm = p.pins.pio0_23.into_input_pin(gpio.tokens.pio0_23);
        let mut pwm_int = pinint
            .interrupts
            .pinint3
            .select::<PIO0_23>(pwm.inner(), &mut syscon.handle);
        pwm_int.enable_rising_edge();
        pwm_int.enable_falling_edge();

        // Configure GPIO pin 5
        let pin_5 = p.pins.pio0_20.into_output_pin(
            gpio.tokens.pio0_20,
            gpio::Level::Low,
        );

        // Configure pin connected to target's input pin
        let red = p.pins.pio1_2.into_output_pin(
            gpio.tokens.pio1_2,
            gpio::Level::High,
        );

        let cts = p.pins.pio0_8.into_output_pin(
            gpio.tokens.pio0_8,
            gpio::Level::Low,
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
        let mut host = p.USART0.enable_async(
            &clock_config,
            &mut syscon.handle,
            u0_rxd,
            u0_txd,
            usart::Settings::default(),
        );
        host.enable_interrupts(usart::Interrupts {
            RXRDY: true,
            .. usart::Interrupts::default()
        });

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
        let mut target = p.USART1.enable_async(
            &clock_config,
            &mut syscon.handle,
            u1_rxd,
            u1_txd,
            usart::Settings::default(),
        );
        target.enable_interrupts(usart::Interrupts {
            RXRDY: true,
            .. usart::Interrupts::default()
        });

        // Configure interrupt for RTS pin
        let rts = p.pins.pio0_9.into_input_pin(gpio.tokens.pio0_9);
        let mut rts_int = pinint
            .interrupts
            .pinint2
            .select(rts.inner(), &mut syscon.handle);
        rts_int.enable_rising_edge();
        rts_int.enable_falling_edge();
        let (rts_int, rts_idle) = RTS.init(rts_int, timers.mrt2);

        // Assign pins to USART2.
        let (u2_rxd, _) = swm.movable_functions.u2_rxd.assign(
            p.pins.pio0_28.into_swm_pin(),
            &mut swm_handle,
        );
        let (u2_txd, _) = swm.movable_functions.u2_txd.assign(
            p.pins.pio0_29.into_swm_pin(),
            &mut swm_handle,
        );

        // Use USART2 as secondary means to communicate with test target.
        let target2 = p.USART2.enable_async(
            &clock_config,
            &mut syscon.handle,
            u2_rxd,
            u2_txd,
            usart::Settings::default(),
        );

        // Assign pins to USART3.
        let (u3_rxd, _) = swm.movable_functions.u3_rxd.assign(
            p.pins.pio0_13.into_swm_pin(),
            &mut swm_handle,
        );
        let (u3_txd, _) = swm.movable_functions.u3_txd.assign(
            p.pins.pio0_14.into_swm_pin(),
            &mut swm_handle,
        );
        let (u3_sclk, _) = swm.movable_functions.u3_sclk.assign(
            p.pins.pio0_15.into_swm_pin(),
            &mut swm_handle,
        );

        // Use USART3 as tertiary means to communicate with the test target.
        let mut target_sync = p.USART3.enable_sync_as_slave(
            &syscon.iosc,
            &mut syscon.handle,
            u3_rxd,
            u3_txd,
            u3_sclk,
            usart::Settings::default(),
        );
        target_sync.enable_interrupts(usart::Interrupts {
            RXRDY: true,
            .. usart::Interrupts::default()
        });

        let (host_rx_int,   host_rx_idle,   host_tx)   = HOST.init(host);
        let (target_rx_int, target_rx_idle, target_tx) = TARGET.init(target);
        let (target_sync_rx_int, target_sync_rx_idle, target_sync_tx) =
            TARGET_SYNC.init(target_sync);

        let (green_int, green_idle) = GREEN.init(green_int, timers.mrt0);
        let (blue_int,  blue_idle)  = BLUE.init(blue_int, timers.mrt1);
        let (pwm_int,   pwm_idle)   = PWM.init(pwm_int, timers.mrt3);

        // Assign I2C0 pin functions
        let (i2c0_sda, _) = swm.fixed_functions.i2c0_sda
            .assign(p.pins.pio0_11.into_swm_pin(), &mut swm_handle);
        let (i2c0_scl, _) = swm.fixed_functions.i2c0_scl
            .assign(p.pins.pio0_10.into_swm_pin(), &mut swm_handle);

        // Initialize I2C0
        let mut i2c = p.I2C0
            .enable(
                &syscon.iosc,
                i2c0_scl,
                i2c0_sda,
                &mut syscon.handle,
            )
            .enable_slave_mode(
                0x48,
            )
            .expect("Not using a valid address");
        i2c.enable_interrupts(i2c::Interrupts {
            slave_pending: true,
            .. i2c::Interrupts::default()
        });

        let (spi0_sck, _) = swm
            .movable_functions
            .spi0_sck
            .assign(p.pins.pio0_16.into_swm_pin(), &mut swm_handle);
        let (spi0_mosi, _) = swm
            .movable_functions
            .spi0_mosi
            .assign(p.pins.pio0_17.into_swm_pin(), &mut swm_handle);
        let (spi0_miso, _) = swm
            .movable_functions
            .spi0_miso
            .assign(p.pins.pio0_18.into_swm_pin(), &mut swm_handle);
        let (spi0_ssel0, _) = swm
            .movable_functions
            .spi0_ssel0
            .assign(p.pins.pio0_19.into_swm_pin(), &mut swm_handle);

        let mut spi = p.SPI0.enable_as_slave(
            &syscon.iosc,
            &mut syscon.handle,
            spi::MODE_0,
            spi0_sck,
            spi0_mosi,
            spi0_miso,
            spi0_ssel0,
        );
        spi.enable_interrupts(spi::Interrupts {
            rx_ready: true,
            .. Default::default()
        });
        spi.enable_interrupts(spi::Interrupts {
            rx_ready: true,
            slave_select_asserted: true,
            slave_select_deasserted: true,
            .. Default::default()
        });

        init::LateResources {
            host_rx_int,
            host_rx_idle,
            host_tx,

            target_rx_int,
            target_rx_idle,
            target_tx,
            target_tx_dma:   target2.tx,
            target_rts_int:  rts_int,
            target_rts_idle: rts_idle,

            target_sync_rx_int,
            target_sync_rx_idle,
            target_sync_tx,

            green_int,
            green_idle,

            blue_int,
            blue_idle,

            pwm_int,
            pwm_idle,

            pin_5,
            red,
            green,
            cts,

            i2c: i2c.slave,
            spi,
        }
    }

    #[idle(
        resources = [
            host_rx_idle,
            host_tx,
            target_rx_idle,
            target_tx,
            target_tx_dma,
            target_sync_rx_idle,
            target_sync_tx,
            green_idle,
            blue_idle,
            pwm_idle,
            target_rts_idle,
            pin_5,
            red,
            green,
            cts,
        ]
    )]
    fn idle(cx: idle::Context) -> ! {
        let host_rx        = cx.resources.host_rx_idle;
        let host_tx        = cx.resources.host_tx;
        let target_rx      = cx.resources.target_rx_idle;
        let target_tx      = cx.resources.target_tx;
        let target_tx_dma  = cx.resources.target_tx_dma;
        let target_sync_rx = cx.resources.target_sync_rx_idle;
        let target_sync_tx = cx.resources.target_sync_tx;
        let green_idle     = cx.resources.green_idle;
        let blue           = cx.resources.blue_idle;
        let pwm            = cx.resources.pwm_idle;
        let rts            = cx.resources.target_rts_idle;
        let pin_5          = cx.resources.pin_5;
        let red            = cx.resources.red;
        let green          = cx.resources.green;
        let cts            = cx.resources.cts;

        let mut pins = FnvIndexMap::<_, _, U8>::new();

        // ensure that the initial level for green is known before the first level change
        let level = match green.is_high() {
            true  => pin::Level::High,
            false => pin::Level::Low,
        };
        pins.insert(InputPin::Green as usize, (level, None)).unwrap();

        let mut buf = [0; 256];

        loop {
            target_rx
                .process_raw(|data| {
                    host_tx.send_message(
                        &AssistantToHost::UsartReceive {
                            mode: UsartMode::Regular,
                            data,
                        },
                        &mut buf,
                    )
                })
                .expect("Error processing USART data");
            target_sync_rx
                .process_raw(|data| {
                    host_tx.send_message(
                        &AssistantToHost::UsartReceive {
                            mode: UsartMode::Sync,
                            data,
                        },
                        &mut buf,
                    )
                })
                .expect("Error processing USART data");

            host_rx
                .process_message(|message| {
                    match message {
                        HostToAssistant::SendUsart {
                            mode: UsartMode::Regular,
                            data,
                        } => {
                            target_tx.send_raw(data)
                        }
                        HostToAssistant::SendUsart {
                            mode: UsartMode::Dma,
                            data,
                        } => {
                            rprintln!("Sending USART message using DMA.");
                            target_tx_dma.bwrite_all(data)
                        }
                        HostToAssistant::SendUsart {
                            mode: UsartMode::FlowControl,
                            data: _,
                        } => {
                            Ok(())
                        }
                        HostToAssistant::SendUsart {
                            mode: UsartMode::Sync,
                            data,
                        } => {
                            target_sync_tx.send_raw(data)
                        }
                        HostToAssistant::SetPin(
                            pin::SetLevel {
                                pin: OutputPin::Pin5,
                                level,
                            }
                        ) => {
                            match level {
                                pin::Level::High => {
                                    pin_5.set_high();
                                }
                                pin::Level::Low => {
                                    pin_5.set_low();
                                }
                            }
                            Ok(())
                        }
                        HostToAssistant::SetPin(
                            pin::SetLevel {
                                pin: OutputPin::Red,
                                level,
                            }
                        ) => {
                            match level {
                                pin::Level::High => {
                                    red.set_high();
                                }
                                pin::Level::Low => {
                                    red.set_low();
                                }
                            }
                            Ok(())
                        }
                        HostToAssistant::SetPin(
                            pin::SetLevel {
                                pin: OutputPin::Cts,
                                level: pin::Level::High,
                            }
                        ) => {
                            rprintln!("Setting CTS HIGH");
                            cts.set_high();
                            Ok(())
                        }
                        HostToAssistant::SetPin(
                            pin::SetLevel {
                                pin: OutputPin::Cts,
                                level: pin::Level::Low,
                            }
                        ) => {
                            rprintln!("Setting CTS LOW");
                            cts.set_low();
                            Ok(())
                        }
                        HostToAssistant::ReadPin(
                            pin::ReadLevel { pin }
                        ) => {
                            let result = pins.get(&(pin as usize))
                                .map(|&(level, period_ms)| {
                                    pin::ReadLevelResult {
                                        pin,
                                        level,
                                        period_ms,
                                    }
                                });

                            host_tx
                                .send_message(
                                    &AssistantToHost::ReadPinResult(result),
                                    &mut buf,
                                )
                                .unwrap();

                            Ok(())
                        }
                    }
                })
                .expect("Error processing host request");
            host_rx.clear_buf();

            handle_pin_interrupt(green_idle, InputPin::Green, &mut pins);
            handle_pin_interrupt(blue,  InputPin::Blue,  &mut pins);
            handle_pin_interrupt(rts,   InputPin::Rts,   &mut pins);
            handle_pin_interrupt(pwm,   InputPin::Pwm,   &mut pins);

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
                    && !green_idle.is_ready();

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

    #[task(binds = PIN_INT6_USART3, resources = [target_sync_rx_int])]
    fn usart3(cx: usart3::Context) {
        cx.resources.target_sync_rx_int.receive()
            .expect("Error receiving from USART3");
    }

    #[task(binds = PIN_INT0, resources = [green_int])]
    fn pinint0(context: pinint0::Context) {
        context.resources.green_int.handle_interrupt();
    }

    #[task(binds = PIN_INT1, resources = [blue_int])]
    fn pinint1(context: pinint1::Context) {
        context.resources.blue_int.handle_interrupt();
    }

    #[task(binds = PIN_INT2, resources = [target_rts_int])]
    fn pinint2(context: pinint2::Context) {
        context.resources.target_rts_int.handle_interrupt();
    }

    #[task(binds = PIN_INT3, resources = [pwm_int])]
    fn pinint3(context: pinint3::Context) {
        context.resources.pwm_int.handle_interrupt();
    }

    #[task(binds = I2C0, resources = [i2c])]
    fn i2c0(context: i2c0::Context) {
        static mut DATA: Option<u8> = None;

        rprintln!("I2C: Handling I2C0 interrupt...");

        match context.resources.i2c.wait() {
            Ok(i2c::slave::State::AddressMatched(i2c)) => {
                rprintln!("I2C: Address matched.");

                i2c.ack().unwrap();

                rprintln!("I2C: Ack'ed address.");
            }
            Ok(i2c::slave::State::RxReady(i2c)) => {
                rprintln!("I2C: Ready to receive.");

                *DATA = Some(i2c.read().unwrap());
                i2c.ack().unwrap();

                rprintln!("I2C: Received and ack'ed.");
            }
            Ok(i2c::slave::State::TxReady(i2c)) => {
                rprintln!("I2C: Ready to transmit.");

                if let Some(data) = *DATA {
                    i2c.transmit(data << 1).unwrap();
                    rprintln!("I2C: Transmitted.");
                }
            }
            Err(nb::Error::WouldBlock) => {
                // I2C not ready; nothing to do
            }
            Err(err) => {
                panic!("I2C error: {:?}", err);
            }
        }
    }

    #[task(binds = SPI0, resources = [spi])]
    fn spi0(context: spi0::Context) {
        static mut ACTIVE: bool = false;

        let spi = context.resources.spi;

        if spi.is_slave_select_asserted() {
            *ACTIVE = true;
        }
        if *ACTIVE {
            if spi.is_ready_to_receive() {
                let data = spi.receive().unwrap();
                block!(spi.transmit(data << 1))
                    .unwrap();
            }
        }
        if spi.is_slave_select_deasserted() {
            *ACTIVE = false;
        }
    }
};


fn handle_pin_interrupt(
    int:  &mut pin_interrupt::Idle,
    pin:  InputPin,
    pins: &mut FnvIndexMap<usize, (pin::Level, Option<u32>), U8>,
) {
    while let Some(event) = int.next() {
        match event {
            pin_interrupt::Event { level, period } => {
                let level = match level {
                    gpio::Level::High => pin::Level::High,
                    gpio::Level::Low  => pin::Level::Low,
                };

                let period_ms = period.map(|value| value / 12_000);
                pins.insert(pin as usize, (level, period_ms)).unwrap();
            }
        }
    }
}
