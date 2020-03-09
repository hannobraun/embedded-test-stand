use std::time::{
    Duration,
    Instant,
};

use lpc845_messages::{
    HostToTarget,
    TargetToHost,
};

use host_lib::conn::{
    Conn,
    ConnReceiveError,
    ConnSendError,
};


/// Test-suite-specific wrapper around `host_lib::Target`
pub struct Target<'r>(&'r mut Conn);

impl<'r> Target<'r> {
    pub(crate) fn new(target: &'r mut Conn) -> Self {
        Self(target)
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
                TargetToHost::UsartReceive(data) => buf.extend(data),
            }
        }
    }
}


#[derive(Debug)]
pub struct TargetUsartSendError(ConnSendError);

#[derive(Debug)]
pub enum TargetUsartWaitError {
    Receive(ConnReceiveError),
    Timeout,
}
