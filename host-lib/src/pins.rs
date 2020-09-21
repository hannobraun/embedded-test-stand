//! API for remotely controlling and monitoring pins on a test node


use serde::Serialize;

use protocol::pin;

use crate::conn::{
    Conn,
    ConnSendError,
};


/// API for remotely controlling and monitoring pins on a test node
///
/// This struct is intended as a building block for higher-level interface for
/// controlling the test nodes of a specific test stand.
pub struct Pins;

impl Pins {
    /// Create a new instance of `Pins`
    pub fn new() -> Self {
        Self
    }

    /// Commands the node to change pin level
    ///
    /// Constructs the command, calls the `wrap` closure to wrap that command
    /// into a message that the node will understand, then sends that message to
    /// the node through `conn`.
    pub fn set_level<Id, M>(&mut self,
        pin: Id,
        level: pin::Level,
        conn: &mut Conn,
    )
        -> Result<(), ConnSendError>
        where
            M: From<pin::SetLevel<Id>> + Serialize,
    {
        let command = pin::SetLevel { pin, level };
        let message: M = command.into();
        conn.send(&message)
    }
}
