use std::time::{
    Duration,
    Instant,
};

use lpc845_messages::{
    HostToTarget,
    Mode,
    PinState,
    TargetToHost,
};

use host_lib::conn::{
    Conn,
    ConnReceiveError,
    ConnSendError,
};


/// The connection to the test target
pub struct Target(pub(crate) Conn);

impl Target {
    /// Instruct the target to set a GPIO pin high
    pub fn set_pin_high(&mut self) -> Result<(), TargetSetPinHighError> {
        self.0.send(&HostToTarget::SetPin(PinState::High))
            .map_err(|err| TargetSetPinHighError(err))
    }

    /// Instruct the target to set a GPIO pin high
    pub fn set_pin_low(&mut self) -> Result<(), TargetSetPinLowError> {
        self.0.send(&HostToTarget::SetPin(PinState::Low))
            .map_err(|err| TargetSetPinLowError(err))
    }

    /// Indicates whether the input pin is set high
    ///
    /// Uses `pin_state` internally.
    pub fn pin_is_high(&mut self) -> Result<bool, TargetPinReadError> {
        let pin_state = self.pin_state(Duration::from_millis(10))?;
        Ok(pin_state == PinState::High)
    }

    /// Indicates whether the input pin is set low
    ///
    /// Uses `pin_state` internally.
    pub fn pin_is_low(&mut self) -> Result<bool, TargetPinReadError> {
        let pin_state = self.pin_state(Duration::from_millis(10))?;
        Ok(pin_state == PinState::Low)
    }

    /// Receives pin state messages to determine current state of pin
    ///
    /// Will wait for pin state messages for a short amount of time. The most
    /// recent one will be used to determine the pin state.
    pub fn pin_state(&mut self, timeout: Duration)
        -> Result<PinState, TargetPinReadError>
    {
        let mut buf   = Vec::new();
        let     start = Instant::now();

        let mut pin_state = None;

        loop {
            if start.elapsed() > timeout {
                break;
            }

            let message = self.0.receive::<TargetToHost>(timeout, &mut buf);
            let message = match message {
                Ok(message) => {
                    message
                }
                Err(err) if err.is_timeout() => {
                    break;
                }
                Err(err) => {
                    return Err(TargetPinReadError::Receive(err));
                }
            };

            match message {
                TargetToHost::PinLevelChanged { level } => {
                    pin_state = Some(level);
                }
                message => {
                    return Err(
                        TargetPinReadError::UnexpectedMessage(
                            format!("{:?}", message)
                        )
                    );
                }
            }
        }

        match pin_state {
            Some(pin_state) => Ok(pin_state),
            None            => Err(TargetPinReadError::Timeout),
        }
    }

    /// Instruct the target to send this message via USART
    pub fn send_usart(&mut self, message: &[u8])
        -> Result<(), TargetUsartSendError>
    {
        self.0.send(&HostToTarget::SendUsart(Mode::Regular, message))
            .map_err(|err| TargetUsartSendError(err))
    }

    /// Instruct the target to send this message via USART using DMA
    pub fn send_usart_dma(&mut self, message: &[u8])
        -> Result<(), TargetUsartSendError>
    {
        self.0.send(&HostToTarget::SendUsart(Mode::Dma, message))
            .map_err(|err| TargetUsartSendError(err))
    }

    /// Wait to receive the provided data via USART
    ///
    /// Returns the receive buffer, once the data was received. Returns an
    /// error, if it times out before that, or an I/O error occurs.
    pub fn wait_for_usart_rx(&mut self, data: &[u8], timeout: Duration)
        -> Result<Vec<u8>, TargetUsartWaitError>
    {
        self.wait_for_usart_rx_inner(data, timeout, Mode::Regular)
    }

    /// Wait to receive the provided data via USART/DMA
    ///
    /// Returns the receive buffer, once the data was received. Returns an
    /// error, if it times out before that, or an I/O error occurs.
    pub fn wait_for_usart_rx_dma(&mut self, data: &[u8], timeout: Duration)
        -> Result<Vec<u8>, TargetUsartWaitError>
    {
        self.wait_for_usart_rx_inner(data, timeout, Mode::Dma)
    }

    fn wait_for_usart_rx_inner(&mut self,
        data:          &[u8],
        timeout:       Duration,
        expected_mode: Mode,
    )
        -> Result<Vec<u8>, TargetUsartWaitError>
    {
        let mut buf   = Vec::new();
        let     start = Instant::now();

        loop {
            if buf.windows(data.len()).any(|window| window == data) {
                return Ok(buf);
            }
            if start.elapsed() > timeout {
                return Err(TargetUsartWaitError::Timeout);
            }

            let mut tmp = Vec::new();
            let message = self.0.receive::<TargetToHost>(timeout, &mut tmp)
                .map_err(|err| TargetUsartWaitError::Receive(err))?;

            match message {
                TargetToHost::UsartReceive(mode, data)
                    if mode == expected_mode =>
                {
                    buf.extend(data)
                }
                message => {
                    return Err(
                        TargetUsartWaitError::UnexpectedMessage(
                            format!("{:?}", message)
                        )
                    );
                }
            }
        }
    }

    /// Start a timer interrupt with the given period in milliseconds
    pub fn start_timer_interrupt(&mut self, period_ms: u32)
        -> Result<TimerInterrupt, TargetStartTimerInterruptError>
    {
        self.0.send(&HostToTarget::StartTimerInterrupt { period_ms })
            .map_err(|err| TargetStartTimerInterruptError(err))?;

        Ok(TimerInterrupt(self))
    }

    /// Start an I2C transaction
    ///
    /// Sends the provided `data` and returns the reply.
    pub fn start_i2c_transaction(&mut self, data: u8, timeout: Duration)
        -> Result<u8, TargetI2cError>
    {
        self.start_i2c_transaction_inner(data, timeout, Mode::Regular)
    }

    /// Start an I2C/DMA transaction
    ///
    /// Sends the provided `data` and returns the reply.
    pub fn start_i2c_transaction_dma(&mut self, data: u8, timeout: Duration)
        -> Result<u8, TargetI2cError>
    {
        self.start_i2c_transaction_inner(data, timeout, Mode::Dma)
    }

    fn start_i2c_transaction_inner(&mut self,
        data:    u8,
        timeout: Duration,
        mode:    Mode,
    )
        -> Result<u8, TargetI2cError>
    {
        let address = 0x48;

        self.0.send(&HostToTarget::StartI2cTransaction { mode, address, data })
            .map_err(|err| TargetI2cError::Send(err))?;

        let mut tmp = Vec::new();
        let message = self.0.receive::<TargetToHost>(timeout, &mut tmp)
            .map_err(|err| TargetI2cError::Receive(err))?;

        match message {
            TargetToHost::I2cReply(reply) => {
                Ok(reply)
            }
            message => {
                Err(
                    TargetI2cError::UnexpectedMessage(
                        format!("{:?}", message)
                    )
                )
            }
        }
    }

    /// Start an SPI transaction
    ///
    /// Sends the provided `data` and returns the reply.
    pub fn start_spi_transaction(&mut self, data: u8, timeout: Duration)
        -> Result<u8, TargetSpiError>
    {
        self.start_spi_transaction_inner(data, timeout, Mode::Regular)
    }

    fn start_spi_transaction_inner(&mut self,
        data:    u8,
        timeout: Duration,
        mode:    Mode,
    )
        -> Result<u8, TargetSpiError>
    {
        self.0.send(&HostToTarget::StartSpiTransaction { mode, data })
            .map_err(|err| TargetSpiError::Send(err))?;

        let mut tmp = Vec::new();
        let message = self.0.receive::<TargetToHost>(timeout, &mut tmp)
            .map_err(|err| TargetSpiError::Receive(err))?;

        match message {
            TargetToHost::SpiReply(reply) => {
                Ok(reply)
            }
            message => {
                Err(
                    TargetSpiError::UnexpectedMessage(
                        format!("{:?}", message)
                    )
                )
            }
        }
    }
}


/// Represent a timer interrupt that's currently configured on the target
///
/// This timer interrupt will be stopped when this struct is dropped.
pub struct TimerInterrupt<'r>(&'r mut Target);

impl Drop for TimerInterrupt<'_> {
    fn drop(&mut self) {
        (self.0).0.send(&HostToTarget::StopTimerInterrupt)
            .unwrap()
    }
}


#[derive(Debug)]
pub struct TargetSetPinHighError(ConnSendError);

#[derive(Debug)]
pub struct TargetSetPinLowError(ConnSendError);

#[derive(Debug)]
pub enum TargetPinReadError {
    Receive(ConnReceiveError),
    Timeout,
    UnexpectedMessage(String),
}


#[derive(Debug)]
pub struct TargetUsartSendError(ConnSendError);

#[derive(Debug)]
pub struct TargetStartTimerInterruptError(ConnSendError);

#[derive(Debug)]
pub enum TargetUsartWaitError {
    Receive(ConnReceiveError),
    Timeout,
    UnexpectedMessage(String),
}

#[derive(Debug)]
pub enum TargetI2cError {
    Send(ConnSendError),
    Receive(ConnReceiveError),
    UnexpectedMessage(String),
}

#[derive(Debug)]
pub enum TargetSpiError {
    Send(ConnSendError),
    Receive(ConnReceiveError),
    UnexpectedMessage(String),
}
