use std::time::{
    Duration,
    Instant,
};

use lpc845_messages::{
    HostToTarget,
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

    /// Instruct the target to send this message via USART
    pub fn send_usart(&mut self, message: &[u8])
        -> Result<(), TargetUsartSendError>
    {
        self.0.send(&HostToTarget::SendUsart(message))
            .map_err(|err| TargetUsartSendError(err))
    }

    /// Wait to receive the provided data via USART
    ///
    /// Returns the receive buffer, once the data was received. Returns an
    /// error, if it times out before that, or an I/O error occurs.
    pub fn wait_for_usart_rx(&mut self, data: &[u8], timeout: Duration)
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

            let mut tmp   = Vec::new();
            let event = self.0.receive::<TargetToHost>(timeout, &mut tmp)
                .map_err(|err| TargetUsartWaitError::Receive(err))?;

            match event {
                TargetToHost::UsartReceive(data) => {
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
pub struct TargetUsartSendError(ConnSendError);

#[derive(Debug)]
pub struct TargetStartTimerInterruptError(ConnSendError);

#[derive(Debug)]
pub enum TargetUsartWaitError {
    Receive(ConnReceiveError),
    Timeout,
    UnexpectedMessage(String),
}
