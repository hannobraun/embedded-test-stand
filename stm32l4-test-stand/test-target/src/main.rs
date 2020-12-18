//! Firmware for the STM32L4 Test Stand


#![no_main]
#![no_std]


extern crate panic_rtt_target;


use cortex_m::peripheral::{
    SYST,
    syst::SystClkSource,
};
use embedded_hal::spi;
use heapless::{
    pool,
    Vec,
    consts::U256,
    pool::singleton::{
        Box,
        Pool as _,
    },
    spsc,
};
use rtt_target::{
    rprint,
    rprintln,
};
use stm32l4xx_hal::{
    prelude::*,
    adc::ADC,
    delay::Delay,
    dma::{
        DMAFrame,
        FrameReader,
        FrameSender,
        dma1,
    },
    gpio::{
        AF4,
        AF5,
        Alternate,
        Analog,
        Floating,
        Input,
        OpenDrain,
        Output,
        PA9,
        PA10,
        PB1,
        PB13,
        PB14,
        PB15,
        PC0,
        PC1,
        PC2,
        PC7,
        PushPull,
    },
    i2c::I2c,
    pac::{
        self,
        I2C1,
        SPI2,
        TIM1,
        USART1,
        USART2,
        USART3,
    },
    pwm::{
        self,
        Pwm,
    },
    rcc::Clocks,
    serial::{
        self,
        Serial,
    },
    spi::Spi,
};

use lpc845_messages::{
    DmaMode,
    HostToTarget,
    TargetToHost,
    UsartMode,
    pin,
};


pool!(
    #[allow(non_upper_case_globals)]
    DmaPool: DMAFrame<U256>
);


#[rtic::app(device = stm32l4xx_hal::pac)]
const APP: () = {
    struct Resources {
        rx_main: serial::Rx<USART1>,
        tx_main: serial::Tx<USART1>,
        rx_host: serial::Rx<USART2>,
        tx_host: serial::Tx<USART2>,
        rx_dma: serial::Rx<USART3>,
        tx_dma: serial::Tx<USART3>,

        rx_prod_main: spsc::Producer<'static, u8, U256>,
        rx_cons_main: spsc::Consumer<'static, u8, U256>,
        rx_prod_host: spsc::Producer<'static, u8, U256>,
        rx_cons_host: spsc::Consumer<'static, u8, U256>,
        rx_prod_dma: spsc::Producer<'static, u8, U256>,
        rx_cons_dma: spsc::Consumer<'static, u8, U256>,

        dma_tx_main: FrameSender<Box<DmaPool>, dma1::C4, U256>,
        dma_rx_dma: FrameReader<Box<DmaPool>, dma1::C3, U256>,

        adc: ADC,
        analog: PC0<Analog>,

        gpio_out: PC1<Output<PushPull>>,
        gpio_in: PC2<Input<Floating>>,

        i2c: I2c<
            I2C1,
            (
                PA9<Alternate<AF4, Output<OpenDrain>>>,
                PA10<Alternate<AF4, Output<OpenDrain>>>
            )
        >,

        ssel: PB1<Output<PushPull>>,
        spi: Spi<
            SPI2,
            (
                PB13<Alternate<AF5, Input<Floating>>>,
                PB14<Alternate<AF5, Input<Floating>>>,
                PB15<Alternate<AF5, Input<Floating>>>,
            )
        >,

        systick: SYST,
        timer_signal: PC7<Output<PushPull>>,
        clocks: Clocks,

        pwm_signal: Pwm<TIM1, pwm::C4>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        static mut RX_QUEUE_HOST: spsc::Queue<u8, U256> =
            spsc::Queue(heapless::i::Queue::new());
        static mut RX_QUEUE_MAIN: spsc::Queue<u8, U256> =
            spsc::Queue(heapless::i::Queue::new());
        static mut RX_QUEUE_DMA: spsc::Queue<u8, U256> =
            spsc::Queue(heapless::i::Queue::new());

        // Allocate memory for DMA transfers.
        static mut MEMORY: [u8; 1024] = [0; 1024];
        DmaPool::grow(MEMORY);

        rtt_target::rtt_init_print!();
        rprint!("Starting target...");

        let cp = cx.core;
        let p = pac::Peripherals::take().unwrap();

        let mut rcc = p.RCC.constrain();
        let mut flash = p.FLASH.constrain();
        let mut pwr = p.PWR.constrain(&mut rcc.apb1r1);

        let clocks = rcc.cfgr
            .pclk1(2.mhz()) // needed to slow down SPI2 clock rate
            .freeze(&mut flash.acr, &mut pwr);

        let mut delay = Delay::new(cp.SYST, clocks);
        let adc = ADC::new(
            p.ADC,
            &mut rcc.ahb2,
            &mut rcc.ccipr,
            &mut delay,
        );
        let systick = delay.free();

        let mut gpioa = p.GPIOA.split(&mut rcc.ahb2);
        let mut gpiob = p.GPIOB.split(&mut rcc.ahb2);
        let mut gpioc = p.GPIOC.split(&mut rcc.ahb2);

        let tx_pin_main = gpiob.pb6
            // Activating AF7 without doing anything else, will cause the pin to
            // go LOW for a moment, which confuses the assistance, causing a
            // panic in the USART1 interrupt handler. I don't know why, but
            // putting it into output mode first prevents this.
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper)
            .into_floating_input(&mut gpiob.moder, &mut gpiob.pupdr)
            .into_af7(&mut gpiob.moder, &mut gpiob.afrl);
        let rx_pin_main = gpiob.pb7.into_af7(&mut gpiob.moder, &mut gpiob.afrl);
        let rts_main = gpiob.pb3.into_af7(&mut gpiob.moder, &mut gpiob.afrl);
        let cts_main = gpiob.pb4.into_af7(&mut gpiob.moder, &mut gpiob.afrl);
        let tx_pin_host = gpioa.pa2.into_af7(&mut gpioa.moder, &mut gpioa.afrl);
        let rx_pin_host = gpioa.pa3.into_af7(&mut gpioa.moder, &mut gpioa.afrl);
        let tx_pin_dma = gpiob.pb10.into_af7(&mut gpiob.moder, &mut gpiob.afrh);
        let rx_pin_dma = gpiob.pb11.into_af7(&mut gpiob.moder, &mut gpiob.afrh);

        let analog = gpioc.pc0.into_analog(&mut gpioc.moder, &mut gpioc.pupdr);

        let gpio_out = gpioc.pc1
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);
        let gpio_in = gpioc.pc2
            .into_floating_input(&mut gpioc.moder, &mut gpioc.pupdr);

        let timer_signal = gpioc.pc7
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);

        let pwm_signal = gpioa.pa11
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper)
            .into_af1(&mut gpioa.moder, &mut gpioa.afrh);
        let pwm_signal = p.TIM1
            .pwm(pwm_signal, 50.hz(), clocks, &mut rcc.apb2);

        let mut scl = gpioa.pa9
            .into_open_drain_output(&mut gpioa.moder, &mut gpioa.otyper);
        scl.internal_pull_up(&mut gpioa.pupdr, true);
        let scl = scl.into_af4(&mut gpioa.moder, &mut gpioa.afrh);
        let mut sda = gpioa.pa10
            .into_open_drain_output(&mut gpioa.moder, &mut gpioa.otyper);
        sda.internal_pull_up(&mut gpioa.pupdr, true);
        let sda = sda.into_af4(&mut gpioa.moder, &mut gpioa.afrh);

        let mut ssel = gpiob.pb1
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
        ssel.set_high().unwrap();
        let sck = gpiob.pb13.into_af5(&mut gpiob.moder, &mut gpiob.afrh);
        let miso = gpiob.pb14.into_af5(&mut gpiob.moder, &mut gpiob.afrh);
        let mosi = gpiob.pb15.into_af5(&mut gpiob.moder, &mut gpiob.afrh);

        let mut usart_main = Serial::usart1(
            p.USART1,
            (tx_pin_main, rx_pin_main, rts_main, cts_main),
            serial::Config::default().baudrate(115_200.bps()),
            clocks,
            &mut rcc.apb2,
        );
        let mut usart_host = Serial::usart2(
            p.USART2,
            (tx_pin_host, rx_pin_host),
            serial::Config::default().baudrate(115_200.bps()),
            clocks,
            &mut rcc.apb1r1,
        );
        let mut usart_dma = Serial::usart3(
            p.USART3,
            (tx_pin_dma, rx_pin_dma),
            serial::Config::default()
                .baudrate(115_200.bps())
                .character_match(b'!'),
            clocks,
            &mut rcc.apb1r1,
        );

        usart_main.listen(serial::Event::Rxne);
        usart_host.listen(serial::Event::Rxne);
        usart_dma.listen(serial::Event::CharacterMatch);

        let i2c = I2c::i2c1(
            p.I2C1,
            (scl, sda),
            100.khz(),
            clocks,
            &mut rcc.apb1r1,
        );

        let spi = Spi::spi2(
            p.SPI2,
            (sck, miso, mosi),
            spi::MODE_0,
            1.khz(),
            clocks,
            &mut rcc.apb1r1,
        );

        let (tx_main, rx_main) = usart_main.split();
        let (tx_host, rx_host) = usart_host.split();
        let (tx_dma, rx_dma) = usart_dma.split();
        let (rx_prod_main, rx_cons_main) = RX_QUEUE_MAIN.split();
        let (rx_prod_host, rx_cons_host) = RX_QUEUE_HOST.split();
        let (rx_prod_dma, rx_cons_dma) = RX_QUEUE_DMA.split();

        let dma1 = p.DMA1.split(&mut rcc.ahb1);
        let dma_tx_main = tx_main.frame_sender(dma1.4);
        let dma_rx_dma = {
            let buf = DmaPool::alloc()
                .unwrap()
                .init(DMAFrame::new());
            rx_dma.frame_read(dma1.3, buf)
        };

        rprintln!("done.");

        init::LateResources {
            rx_main,
            tx_main,
            rx_host,
            tx_host,
            rx_dma,
            tx_dma,

            rx_prod_main,
            rx_cons_main,
            rx_prod_host,
            rx_cons_host,
            rx_prod_dma,
            rx_cons_dma,

            dma_tx_main,
            dma_rx_dma,

            adc,
            analog,

            gpio_out,
            gpio_in,

            i2c,

            ssel,
            spi,

            systick,
            timer_signal,
            clocks,

            pwm_signal,
        }
    }

    #[idle(resources = [
        rx_cons_main,
        rx_cons_host,
        rx_cons_dma,
        tx_main,
        tx_host,
        dma_tx_main,
        adc,
        analog,
        gpio_out,
        gpio_in,
        i2c,
        ssel,
        spi,
        systick,
        clocks,
        pwm_signal,
    ])]
    fn idle(cx: idle::Context) -> ! {
        let rx_main = cx.resources.rx_cons_main;
        let rx_host = cx.resources.rx_cons_host;
        let rx_dma  = cx.resources.rx_cons_dma;
        let tx_main = cx.resources.tx_main;
        let tx_host = cx.resources.tx_host;
        let dma_tx_main = cx.resources.dma_tx_main;
        let adc = cx.resources.adc;
        let analog = cx.resources.analog;
        let gpio_out = cx.resources.gpio_out;
        let gpio_in = cx.resources.gpio_in;
        let i2c = cx.resources.i2c;
        let ssel = cx.resources.ssel;
        let spi = cx.resources.spi;
        let systick = cx.resources.systick;
        let clocks = cx.resources.clocks;
        let pwm_signal = cx.resources.pwm_signal;

        let mut buf_main_rx: Vec<_, U256> = Vec::new();
        let mut buf_host_rx: Vec<_, U256> = Vec::new();

        loop {
            handle_usart_rx(
                rx_main,
                tx_host,
                UsartMode::Regular,
                &mut buf_main_rx,
            );
            handle_usart_rx(
                rx_dma,
                tx_host,
                UsartMode::Dma,
                &mut buf_main_rx,
            );

            if let Some(b) = rx_host.dequeue() {
                // Requests are COBS-encoded, so we know that `0` means we
                // received a full frame.
                if b != 0 {
                    buf_host_rx.push(b).expect("Receive buffer full");
                    continue;
                }

                let message = postcard::from_bytes_cobs(&mut buf_host_rx)
                    .expect("Error decoding message");
                match message {
                    HostToTarget::SendUsart {
                        mode: UsartMode::Regular,
                        data,
                    } => {
                        tx_main.bwrite_all(data)
                            .expect("Error writing to USART");
                        rprintln!("Sent data from host: {:?}", data);
                    }
                    HostToTarget::SendUsart {
                        mode: UsartMode::Dma,
                        data,
                    } => {
                        rprint!("Sending using USART/DMA...");

                        let buf = DmaPool::alloc().unwrap();
                        let mut buf = buf.init(DMAFrame::new());
                        buf.write_slice(data);

                        dma_tx_main.send(buf).unwrap();

                        loop {
                            let buf = dma_tx_main.transfer_complete_interrupt();
                            if let Some(buf) = buf {
                                // Not sure why, but the buffer needs to be
                                // dropped explicitly for its memory to be
                                // freed.
                                drop(buf);
                                break;
                            }
                        }

                        rprintln!("done.")
                    }
                    HostToTarget::SendUsart {
                        mode: UsartMode::FlowControl,
                        data,
                    } => {
                        // Re-using USART1 for the flow control test.
                        // Unfortunately the STM32L433 doesn't have enough
                        // USARTs to test this on a separate instance.
                        tx_main.bwrite_all(data)
                            .expect("Error writing to USART");

                        rprintln!("Sent data using flow control: {:?}", data);
                    }
                    HostToTarget::ReadAdc => {
                        let value = adc.read(analog).unwrap();

                        let message = TargetToHost::AdcValue(value);

                        let buf_host_tx: Vec<_, U256> =
                            postcard::to_vec_cobs(&message)
                                .expect("Error encoding message to host");
                        tx_host.bwrite_all(buf_host_tx.as_ref())
                            .expect("Error sending message to host");
                    }
                    HostToTarget::SetPin(
                        pin::SetLevel { level, pin: () }
                    ) => {
                        match level {
                            pin::Level::High => {
                                gpio_out.set_high().unwrap();
                            }
                            pin::Level::Low => {
                                gpio_out.set_low().unwrap();
                            }
                        }
                    }
                    HostToTarget::ReadPin(pin::ReadLevel { pin: () }) => {
                        let level = match gpio_in.is_high().unwrap() {
                            true  => pin::Level::High,
                            false => pin::Level::Low,
                        };

                        let message = TargetToHost::ReadPinResult(
                            Some(
                                pin::ReadLevelResult {
                                    pin: (),
                                    level,
                                    period_ms: None,
                                }
                            )
                        );

                        let buf_host_tx: Vec<_, U256> =
                            postcard::to_vec_cobs(&message)
                                .expect("Error encoding message to host");
                        tx_host.bwrite_all(buf_host_tx.as_ref())
                            .expect("Error sending message to host");
                    }
                    HostToTarget::StartI2cTransaction {
                        mode: DmaMode::Regular,
                        address,
                        data,
                    } => {
                        i2c.write(address, &[data])
                            .unwrap();

                        let mut rx_buf = [0u8; 1];
                        i2c.read(address, &mut rx_buf)
                            .unwrap();

                        let message = TargetToHost::I2cReply(rx_buf[0]);

                        let buf_host_tx: Vec<_, U256> =
                            postcard::to_vec_cobs(&message)
                                .expect("Error encoding message to host");
                        tx_host.bwrite_all(buf_host_tx.as_ref())
                            .expect("Error sending message to host");
                    }
                    HostToTarget::StartSpiTransaction {
                        mode: DmaMode::Regular,
                        data,
                    } => {
                        rprintln!("SPI: Set SSEL LOW");
                        ssel.set_low().unwrap();

                        let mut data = [data, 0xFF];
                        spi.transfer(&mut data).unwrap();
                        let reply = data[1];

                        rprintln!("SPI: Set SSEL HIGH");
                        ssel.set_high().unwrap();

                        let message = TargetToHost::SpiReply(reply);

                        let buf_host_tx: Vec<_, U256> =
                            postcard::to_vec_cobs(&message)
                                .expect("Error encoding message to host");
                        tx_host.bwrite_all(buf_host_tx.as_ref())
                            .expect("Error sending message to host");

                        rprintln!(" done.");
                    }
                    HostToTarget::StartTimerInterrupt { period_ms } => {
                        let reload = clocks.hclk().0 / 1000 * period_ms;
                        systick.set_clock_source(SystClkSource::Core);
                        systick.set_reload(reload);

                        systick.clear_current();
                        systick.enable_interrupt();
                        systick.enable_counter();
                    }
                    HostToTarget::StopTimerInterrupt => {
                        systick.disable_interrupt();
                        systick.disable_counter();
                    }
                    HostToTarget::StartPwmSignal => {
                        pwm_signal.set_duty(pwm_signal.get_max_duty() / 2);
                        pwm_signal.enable();
                    }
                    HostToTarget::StopPwmSignal => {
                        pwm_signal.disable();
                    }
                    message => {
                        panic!("Unsupported message: {:?}", message)
                    }
                }

                buf_host_rx.clear();
            }
        }
    }

    #[task(binds = USART1, resources = [rx_main, rx_prod_main])]
    fn usart1(cx: usart1::Context) {
        let rx = cx.resources.rx_main;
        let queue = cx.resources.rx_prod_main;

        let b = match rx.read() {
            Ok(b) => b,
            Err(err) => {
                rprintln!("Error reading from USART1: {:?}", err);
                return;
            }
        };
        match queue.enqueue(b) {
            Ok(()) => (),
            Err(err) => {
                rprintln!("Error adding received byte to queue: {:?}", err);
                return;
            }
        }
    }

    #[task(binds = USART2, resources = [rx_host, rx_prod_host])]
    fn usart2(cx: usart2::Context) {
        let rx = cx.resources.rx_host;
        let queue = cx.resources.rx_prod_host;

        let b = match rx.read() {
            Ok(b) => b,
            Err(err) => {
                rprintln!("Error reading from USART2: {:?}", err);
                return;
            }
        };
        match queue.enqueue(b) {
            Ok(()) => (),
            Err(err) => {
                rprintln!("Error adding received byte to queue: {:?}", err);
                return;
            }
        }
    }

    #[task(binds = USART3, resources = [rx_dma, dma_rx_dma, rx_prod_dma])]
    fn usart3(cx: usart3::Context) {
        let rx_dma = cx.resources.rx_dma;
        let dma_rx_dma = cx.resources.dma_rx_dma;
        let rx_prod_dma = cx.resources.rx_prod_dma;

        if rx_dma.is_character_match(true) {
            let buf = DmaPool::alloc()
                .unwrap()
                .init(DMAFrame::new());
            let buf = dma_rx_dma.character_match_interrupt(buf);

            for &b in buf.read() {
                rx_prod_dma.enqueue(b).unwrap();
            }
        }
    }

    #[task(binds = SysTick, resources = [timer_signal])]
    fn syst(cx: syst::Context) {
        // cx.resources.timer_signal.toggle();
        static mut HIGH: bool = false;

        let timer_signal = cx.resources.timer_signal;

        if *HIGH {
            timer_signal.set_low().unwrap();
        }
        else {
            timer_signal.set_high().unwrap();
        }

        *HIGH = !*HIGH;
    }
};

fn handle_usart_rx(
    queue: &mut spsc::Consumer<'static, u8, U256>,
    tx_host: &mut serial::Tx<USART2>,
    mode: UsartMode,
    buf: &mut Vec<u8, U256>,
) {
    while let Some(b) = queue.dequeue() {
        buf.push(b)
            .expect("Main receive buffer full");
    }

    if buf.len() > 0 {
        let message = TargetToHost::UsartReceive {
            mode,
            data: buf.as_ref(),
        };

        let buf_host_tx: Vec<_, U256> = postcard::to_vec_cobs(&message)
            .expect("Error encoding message to host");
        tx_host.bwrite_all(buf_host_tx.as_ref())
            .expect("Error sending message to host");

        buf.clear();
    }
}
