//! API for remotely controlling and monitoring pins on a test node


use std::{
    convert::TryInto,
    fmt::Debug,
    mem::transmute,
    thread::sleep,
    time::Duration,
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


/// API for remotely controlling and monitoring a pin on a test node
///
/// This struct is intended as a building block for higher-level interfaces
/// that control the test nodes of a specific test stand.
pub struct Pin<Id> {
    pin: Id,
}

impl<Id> Pin<Id>
    where Id: Copy
{
    /// Create a new instance of `Pins`
    pub fn new(pin: Id) -> Self {
        Self {
            pin,
        }
    }

    /// Commands the node to change pin level
    ///
    /// Constructs the command, calls the `wrap` closure to wrap that command
    /// into a message that the node will understand, then sends that message to
    /// the node through `conn`.
    pub fn set_level<M>(&mut self,
        level: pin::Level,
        conn: &mut Conn,
    )
        -> Result<(), ConnSendError>
        where
            M: From<pin::SetLevel<Id>> + Serialize,
    {
        let command = pin::SetLevel { pin: self.pin, level };
        let message: M = command.into();
        conn.send(&message)?;

        Ok(())
    }

    /// Read level for the given pin
    ///
    /// Receives from `conn`, expecting to receive a "level changed" message.
    /// Uses `unwrap` to get a `pin::LevelChange` from the message.
    pub fn read_level<'de, Request, Reply>(&mut self,
        timeout: Duration,
        conn: &mut Conn,
    )
        -> Result<(pin::Level, Option<u32>), ReadLevelError>
        where
            Id: Debug + Eq,
            Request: From<pin::ReadLevel<Id>> + Serialize,
            Reply: TryInto<pin::ReadLevelResult<Id>, Error=Reply>
                + Debug
                + Deserialize<'de>,
    {
        // Wait for a bit, to give whatever event is expected to change the
        // level some time to happen.
        sleep(timeout);

        let request = pin::ReadLevel {  pin: self.pin };
        let request: Request = request.into();
        conn.send(&request)
            .map_err(|err| ReadLevelError::Send(err))?;

        // The compiler believes that `buf` doesn't live long enough, because
        // the lifetime of the buffer needs to be `'de`, due to the
        // `Deserialize` bound on `Reply`. This is wrong though: Nothing we
        // return from this method still references the buffer, so the following
        // `transmute`, which transmutes a mutable reference to `buf` to a
        // mutable reference with unbounded lifetime, is sound.
        let mut buf: Vec<u8> = Vec::new();
        let buf = unsafe { transmute(&mut buf) };

        let reply = conn.receive::<Reply>(timeout, buf)
            .map_err(|err| ReadLevelError::Receive(err))?;

        match reply.try_into() {
            Ok(
                pin::ReadLevelResult {
                    pin,
                    level,
                    period_ms,
                }
            )
                if pin == self.pin
            => {
                Ok((level, period_ms))
            }
            Err(message) => {
                Err(
                    ReadLevelError::UnexpectedMessage(
                        format!("{:?}", message)
                    )
                )
            }
            message => {
                Err(
                    ReadLevelError::UnexpectedMessage(
                        format!("{:?}", message)
                    )
                )
            }
        }
    }
}


#[derive(Debug)]
pub enum ReadLevelError {
    Send(ConnSendError),
    Receive(ConnReceiveError),
    UnexpectedMessage(String),
    Timeout,
}
