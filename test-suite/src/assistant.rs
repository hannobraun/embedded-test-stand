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
    HostToAssistant,
    AssistantToHost,
};


/// The connection to the test assistant
pub struct Assistant<'r>(pub(crate) &'r mut Conn);

impl<'r> Assistant<'r> {
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
                AssistantToHost::UsartReceive(data) => buf.extend(data),
            }
        }
    }
}


#[derive(Debug)]
pub struct AssistantUsartSendError(ConnSendError);

#[derive(Debug)]
pub enum AssistantUsartWaitError {
    Receive(ConnReceiveError),
    Timeout,
}

