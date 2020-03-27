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
    Pin,
};


/// The connection to the test assistant
pub struct Assistant(pub(crate) Conn);

impl Assistant {
    /// Indicates whether the GPIO pin on the test target is set high
    ///
    /// Uses `pin_state` internally.
    pub fn pin_is_high(&mut self) -> Result<bool, AssistantPinReadError> {
        let pin_state = self.pin_state(Pin::Green, Duration::from_millis(10))?;
        Ok(pin_state == PinState::High)
    }

    /// Indicates whether the GPIO pin on the test target is set low
    ///
    /// Uses `pin_state` internally.
    pub fn pin_is_low(&mut self) -> Result<bool, AssistantPinReadError> {
        let pin_state = self.pin_state(Pin::Green, Duration::from_millis(10))?;
        Ok(pin_state == PinState::Low)
    }

    /// Receives pin state messages to determine current state of pin
    ///
    /// Will wait for pin state messages for a short amount of time. The most
    /// recent one will be used to determine the pin state.
    pub fn pin_state(&mut self, pin: Pin, timeout: Duration)
        -> Result<PinState, AssistantPinReadError>
    {
        let mut buf   = Vec::new();
        let     start = Instant::now();

        let mut pin_state = None;

        loop {
            if start.elapsed() > timeout {
                break;
            }

            let message = self.0.receive::<AssistantToHost>(timeout, &mut buf);
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
                AssistantToHost::PinIsHigh(p) if p == pin => {
                    pin_state = Some(PinState::High);
                }
                AssistantToHost::PinIsLow(p) if p == pin => {
                    pin_state = Some(PinState::Low);
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

    /// Instruct assistant to send this message to the target via USART
    pub fn send_to_target_usart(&mut self, message: &[u8])
        -> Result<(), AssistantUsartSendError>
    {
        self.0.send(&HostToAssistant::SendUsart(message))
            .map_err(|err| AssistantUsartSendError(err))
    }

    /// Wait to receive the provided data via USART
    ///
    /// Returns the receive buffer, once the data was received. Returns an
    /// error, if it times out before that, or an I/O error occurs.
    pub fn receive_from_target_usart(&mut self, data: &[u8], timeout: Duration)
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
            let message = self.0.receive::<AssistantToHost>(timeout, &mut tmp)
                .map_err(|err| AssistantUsartWaitError::Receive(err))?;

            match message {
                AssistantToHost::UsartReceive(data) => {
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
}


#[derive(Debug, Eq, PartialEq)]
pub enum PinState {
    High,
    Low,
}


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
