use std::time::{
    Duration,
    Instant,
};

use lpc845_messages::{
    Event,
    Request,
};

use host_lib::target::{
    TargetInitError,
    TargetReceiveError,
    TargetSendError,
};


/// Test-suite-specific wrapper around `host_lib::Target`
pub struct Target(host_lib::Target);

impl Target {
    /// Open a connection to the target
    pub fn new(path: &str) -> Result<Self, TargetInitError> {
        let target = host_lib::Target::new(path)?;
        Ok(Self(target))
    }

    /// Instruct the target to send this message via USART
    pub fn send_usart(&mut self, message: &[u8])
        -> Result<(), TargetSendError>
    {
        self.0.send(&Request::SendUsart(message))
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
            let event = self.0.receive::<Event>(timeout, &mut tmp)
                .map_err(|err| TargetUsartWaitError::Receive(err))?;

            match event {
                Event::UsartReceive(data) => buf.extend(data),
            }
        }
    }
}


#[derive(Debug)]
pub enum TargetUsartWaitError {
    Receive(TargetReceiveError),
    Timeout,
}
