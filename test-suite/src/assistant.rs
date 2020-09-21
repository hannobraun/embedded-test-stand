use std::time::{
    Duration,
    Instant,
};

use host_lib::conn::{
    Conn,
    ConnReceiveError,
    ConnSendError,
};
use lpc845_messages::{
    AssistantToHost,
    HostToAssistant,
    InputPin,
    OutputPin,
    UsartMode,
    pin,
};


/// The connection to the test assistant
pub struct Assistant {
    conn: Conn,
}

impl Assistant {
    pub(crate) fn new(conn: Conn) -> Self {
        Self {
            conn,
        }
    }

    /// Instruct the assistant to set the target's input pin high
    pub fn set_pin_high(&mut self) -> Result<(), AssistantSetPinHighError> {
        self.conn
            .send(&HostToAssistant::SetPin(
                pin::SetLevel {
                    pin: OutputPin::Red,
                    level: pin::Level::High,
                }
            ))
            .map_err(|err| AssistantSetPinHighError(err))
    }

    /// Instruct the assistant to set the target's input pin low
    pub fn set_pin_low(&mut self) -> Result<(), AssistantSetPinLowError> {
        self.conn
            .send(&HostToAssistant::SetPin(
                pin::SetLevel {
                    pin: OutputPin::Red,
                    level: pin::Level::Low,
                }
            ))
            .map_err(|err| AssistantSetPinLowError(err))
    }

    /// Instruct the assistant to disable CTS
    pub fn disable_cts(&mut self) -> Result<(), AssistantSetPinHighError> {
        self.conn
            .send(&HostToAssistant::SetPin(
                pin::SetLevel {
                    pin: OutputPin::Cts,
                    level: pin::Level::High,
                }
            ))
            .map_err(|err| AssistantSetPinHighError(err))
    }

    /// Instruct the assistant to enable CTS
    pub fn enable_cts(&mut self) -> Result<(), AssistantSetPinLowError> {
        self.conn
            .send(&HostToAssistant::SetPin(
                pin::SetLevel {
                    pin: OutputPin::Cts,
                    level: pin::Level::Low,
                }
            ))
            .map_err(|err| AssistantSetPinLowError(err))
    }

    /// Indicates whether the GPIO pin on the test target is set high
    ///
    /// Uses `pin_state` internally.
    pub fn pin_is_high(&mut self) -> Result<bool, AssistantPinReadError> {
        let pin_state = self.pin_state(
            InputPin::Green,
            Duration::from_millis(10),
        )?;
        Ok(pin_state.0 == pin::Level::High)
    }

    /// Indicates whether the GPIO pin on the test target is set low
    ///
    /// Uses `pin_state` internally.
    pub fn pin_is_low(&mut self) -> Result<bool, AssistantPinReadError> {
        let pin_state = self.pin_state(
            InputPin::Green,
            Duration::from_millis(10),
        )?;
        Ok(pin_state.0 == pin::Level::Low)
    }

    /// Receives pin state messages to determine current state of pin
    ///
    /// Will wait for pin state messages for a short amount of time. The most
    /// recent one will be used to determine the pin state.
    pub fn pin_state(&mut self, expected_pin: InputPin, timeout: Duration)
        -> Result<(pin::Level, Option<u32>), AssistantPinReadError>
    {
        let mut buf   = Vec::new();
        let     start = Instant::now();

        let mut pin_state = None;

        loop {
            if start.elapsed() > timeout {
                break;
            }

            let message = self.conn
                .receive::<AssistantToHost>(timeout, &mut buf);
            let message = match message {
                Ok(message) => {
                    message
                }
                Err(err) if err.is_timeout() => {
                    break;
                }
                Err(err) => {
                    return Err(AssistantPinReadError::Receive(err));
                }
            };

            match message {
                AssistantToHost::PinLevelChanged(pin::LevelChanged {
                    pin, level, period_ms
                })
                    if pin == expected_pin
                => {
                    pin_state = Some((level, period_ms));
                }

                _ => {
                    return Err(
                        AssistantPinReadError::UnexpectedMessage(
                            format!("{:?}", message)
                        )
                    );
                }
            }
        }

        match pin_state {
            Some(pin_state) => Ok(pin_state),
            None            => Err(AssistantPinReadError::Timeout),
        }
    }

    /// Wait for RTS signal to be enabled
    pub fn wait_for_rts(&mut self) -> Result<bool, AssistantPinReadError> {
        let pin_state = self.pin_state(
            InputPin::Rts,
            Duration::from_millis(10),
        )?;
        Ok(pin_state.0 == pin::Level::Low)
    }

    /// Instruct assistant to send this message to the target via USART
    pub fn send_to_target_usart(&mut self, data: &[u8])
        -> Result<(), AssistantUsartSendError>
    {
        self.conn
            .send(&HostToAssistant::SendUsart {
                mode: UsartMode::Regular,
                data,
            })
            .map_err(|err| AssistantUsartSendError(err))
    }

    /// Instruct assistant to send this message to the target's USART/DMA
    pub fn send_to_target_usart_dma(&mut self, data: &[u8])
        -> Result<(), AssistantUsartSendError>
    {
        self.conn
            .send(&HostToAssistant::SendUsart { mode: UsartMode::Dma, data })
            .map_err(|err| AssistantUsartSendError(err))
    }

    /// Instruct assistant to send this message to the target's sync USART
    pub fn send_to_target_usart_sync(&mut self, data: &[u8])
        -> Result<(), AssistantUsartSendError>
    {
        self.conn
            .send(&HostToAssistant::SendUsart { mode: UsartMode::Sync, data })
            .map_err(|err| AssistantUsartSendError(err))
    }

    /// Wait to receive the provided data via USART
    ///
    /// Returns the receive buffer, once the data was received. Returns an
    /// error, if it times out before that, or an I/O error occurs.
    pub fn receive_from_target_usart(&mut self, data: &[u8], timeout: Duration)
        -> Result<Vec<u8>, AssistantUsartWaitError>
    {
        self.receive_from_target_usart_inner(data, timeout, UsartMode::Regular)
    }

    /// Wait to receive the provided data via USART in synchronous mode
    ///
    /// Returns the receive buffer, once the data was received. Returns an
    /// error, if it times out before that, or an I/O error occurs.
    pub fn receive_from_target_usart_sync(&mut self,
        data:    &[u8],
        timeout: Duration,
    )
        -> Result<Vec<u8>, AssistantUsartWaitError>
    {
        self.receive_from_target_usart_inner(data, timeout, UsartMode::Sync)
    }

    pub fn receive_from_target_usart_inner(&mut self,
        data:          &[u8],
        timeout:       Duration,
        expected_mode: UsartMode,
    )
        -> Result<Vec<u8>, AssistantUsartWaitError>
    {
        let mut buf   = Vec::new();
        let     start = Instant::now();

        loop {
            if buf.windows(data.len()).any(|window| window == data) {
                return Ok(buf);
            }
            if start.elapsed() > timeout {
                return Err(AssistantUsartWaitError::Timeout);
            }

            let mut tmp = Vec::new();
            let message = self.conn
                .receive::<AssistantToHost>(timeout, &mut tmp)
                .map_err(|err| AssistantUsartWaitError::Receive(err))?;

            match message {
                AssistantToHost::UsartReceive { mode, data }
                    if mode == expected_mode
                => {
                    buf.extend(data)
                }
                _ => {
                    return Err(
                        AssistantUsartWaitError::UnexpectedMessage(
                            format!("{:?}", message)
                        )
                    );
                }
            }
        }
    }

    /// Measures the period of changes in a GPIO signal
    ///
    /// Waits for changes in the GPIO signal until the given number of samples
    /// has been measured. Returns the minimum and maximum period measured, in
    /// milliseconds.
    ///
    /// # Panics
    ///
    /// `samples` must be at least `1`. This method will panic, if this is not
    /// the case.
    pub fn measure_gpio_period(&mut self, samples: u32, timeout: Duration)
        -> Result<GpioPeriodMeasurement, AssistantPinReadError>
    {
        assert!(samples > 0);

        let mut measurement: Option<GpioPeriodMeasurement> = None;

        let (mut state, _) = self.pin_state(InputPin::Blue, timeout)?;

        for _ in 0 .. samples {
            let (new_state, period_ms) = self.pin_state(
                InputPin::Blue,
                timeout,
            )?;
            print!("{:?}, {:?}\n", new_state, period_ms);

            if new_state == state {
                continue;
            }

            state = new_state;

            let period = match period_ms {
                Some(period_ms) => Duration::from_millis(period_ms as u64),
                None            => continue,
            };

            match &mut measurement {
                Some(measurement) => {
                    measurement.min = Ord::min(measurement.min, period);
                    measurement.max = Ord::max(measurement.max, period);
                }
                None => {
                    measurement = Some(
                        GpioPeriodMeasurement {
                            min: period,
                            max: period,
                        }
                    )
                }
            }
        }

        // Due to the assertion above, we know that samples is at least `1` and
        // therefore, that the loop ran at least once. `measurement` must be
        // `Some`.
        Ok(measurement.unwrap())
    }

    /// Expect to hear nothing from the target within the given timeout period
    pub fn expect_nothing_from_target(&mut self, timeout: Duration)
        -> Result<(), AssistantExpectNothingError>
    {
        loop {
            let mut tmp = Vec::new();
            let message = self.conn
                .receive::<AssistantToHost>(timeout, &mut tmp);

            match message {
                Ok(message) => {
                    return Err(
                        AssistantExpectNothingError::UnexpectedMessage(
                            format!("{:?}", message)
                        )
                    );
                }
                Err(err) if err.is_timeout() => {
                    break;
                }
                Err(err) => {
                    return Err(AssistantExpectNothingError::Receive(err));
                }
            }
        }

        Ok(())
    }
}


#[derive(Debug)]
pub struct GpioPeriodMeasurement {
    pub min: Duration,
    pub max: Duration,
}


#[derive(Debug)]
pub struct AssistantSetPinHighError(ConnSendError);

#[derive(Debug)]
pub struct AssistantSetPinLowError(ConnSendError);

#[derive(Debug)]
pub enum AssistantPinReadError {
    Receive(ConnReceiveError),
    UnexpectedMessage(String),
    Timeout,
}

#[derive(Debug)]
pub struct AssistantUsartSendError(ConnSendError);

#[derive(Debug)]
pub enum AssistantUsartWaitError {
    Receive(ConnReceiveError),
    Timeout,
    UnexpectedMessage(String),
}

#[derive(Debug)]
pub enum AssistantExpectNothingError {
    Receive(ConnReceiveError),
    UnexpectedMessage(String),
}
