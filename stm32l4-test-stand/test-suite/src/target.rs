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
    HostToTarget,
    TargetToHost,
    UsartMode,
};


/// The connection to the test target
pub struct Target {
    conn: Conn,
}

impl Target {
    pub(crate) fn new(conn: Conn) -> Self {
        Self {
            conn,
        }
    }

    /// Instruct the target to send this message via USART
    pub fn send_usart(&mut self, data: &[u8])
        -> Result<(), TargetUsartSendError>
    {
        self.conn
            .send(&HostToTarget::SendUsart { mode: UsartMode::Regular, data })
            .map_err(|err| TargetUsartSendError(err))
    }

    /// Instruct the target to send this message via USART using DMA
    pub fn send_usart_dma(&mut self, data: &[u8])
        -> Result<(), TargetUsartSendError>
    {
        self.conn
            .send(&HostToTarget::SendUsart { mode: UsartMode::Dma, data })
            .map_err(|err| TargetUsartSendError(err))
    }

    /// Wait to receive the provided data via USART
    ///
    /// Returns the receive buffer, once the data was received. Returns an
    /// error, if it times out before that, or an I/O error occurs.
    pub fn wait_for_usart_rx(&mut self, data: &[u8], timeout: Duration)
        -> Result<Vec<u8>, TargetUsartWaitError>
    {
        self.wait_for_usart_rx_inner(data, timeout, UsartMode::Regular)
    }

    /// Wait to receive the provided data via USART/DMA
    ///
    /// Returns the receive buffer, once the data was received. Returns an
    /// error, if it times out before that, or an I/O error occurs.
    pub fn wait_for_usart_rx_dma(&mut self, data: &[u8], timeout: Duration)
        -> Result<Vec<u8>, TargetUsartWaitError>
    {
        self.wait_for_usart_rx_inner(data, timeout, UsartMode::Dma)
    }

    fn wait_for_usart_rx_inner(&mut self,
        data:          &[u8],
        timeout:       Duration,
        expected_mode: UsartMode,
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
            let message = self.conn
                .receive::<TargetToHost>(timeout, &mut tmp)
                .map_err(|err| TargetUsartWaitError::Receive(err))?;

            match message {
                TargetToHost::UsartReceive { mode, data }
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
}


#[derive(Debug)]
pub struct TargetUsartSendError(ConnSendError);

#[derive(Debug)]
pub enum TargetUsartWaitError {
    Receive(ConnReceiveError),
    Timeout,
    UnexpectedMessage(String),
}
