//! API for remotely controlling and monitoring pins on a test node


use std::{
    convert::TryInto,
    fmt::Debug,
    mem::transmute,
    time::{
        Duration,
        Instant,
    },
};

use serde::{
    Deserialize,
    Serialize,
};

use protocol::pin;

use crate::conn::{
    Conn,
    ConnReceiveError,
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

    /// Read level for the given pin
    ///
    /// Receives from `conn`, expecting to receive a "level changed" message.
    /// Uses `unwrap` to get a `pin::LevelChange` from the message.
    pub fn read_level<'de, Id, M>(&mut self,
        expected_pin: Id,
        timeout: Duration,
        conn: &mut Conn,
    )
        -> Result<(pin::Level, Option<u32>), ReadLevelError>
        where
            Id: Debug + Eq,
            M: TryInto<pin::LevelChanged<Id>, Error=M>
                + Debug
                + Deserialize<'de>,
    {
        let mut buf: Vec<u8> = Vec::new();

        let     start     = Instant::now();
        let mut pin_level = None;

        loop {
            // Because of the lifetime `'de`, Rust throws an error when we try
            // to borrow `buf` in the loop. What Rust doesn't understand is that
            // the borrow doesn't actually last beyond the loop iteration
            // though.
            //
            // Let's circumvent the borrow checker by creating a lifetime it
            // won't complain about. This is sound, as long as nothing that
            // borrows `buf` lasts beyond the loop iteration.
            let buf = unsafe { transmute(&mut buf) };

            if start.elapsed() > timeout {
                break;
            }

            let message = conn
                .receive::<M>(timeout, buf);
            let message = match message {
                Ok(message) => {
                    message
                }
                Err(err) if err.is_timeout() => {
                    break;
                }
                Err(err) => {
                    return Err(ReadLevelError::Receive(err));
                }
            };

            match message.try_into() {
                Ok(
                    pin::LevelChanged {
                        pin,
                        level,
                        period_ms,
                    }
                )
                    if pin == expected_pin
                => {
                    pin_level = Some((level, period_ms));
                }
                Err(message) => {
                    return Err(
                        ReadLevelError::UnexpectedMessage(
                            format!("{:?}", message)
                        )
                    );
                }
                message => {
                    return Err(
                        ReadLevelError::UnexpectedMessage(
                            format!("{:?}", message)
                        )
                    );
                }
            }
        }

        match pin_level {
            Some(pin_level) => Ok(pin_level),
            None            => Err(ReadLevelError::Timeout),
        }
    }
}


#[derive(Debug)]
pub enum ReadLevelError {
    Receive(ConnReceiveError),
    UnexpectedMessage(String),
    Timeout,
}
