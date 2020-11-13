use host_lib::conn::{
    Conn,
    ConnSendError,
};


use lpc845_messages::{
    HostToTarget,
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
}


#[derive(Debug)]
pub struct TargetUsartSendError(ConnSendError);
