use lpc8xx_hal::{
    prelude::*,
    usart,
};

use serde::Serialize;
use void::ResultVoidExt;

use crate::Result;


/// Sends messages from the firmware to the host
pub struct Sender<'r, USART: usart::Instance, Buf> {
    usart: &'r mut usart::Tx<USART>,
    buf:   Buf,
}

impl<'r, USART, Buf> Sender<'r, USART, Buf>
    where
        USART: usart::Instance,
        Buf:   AsMut<[u8]>,
{
    /// Create a new instance of `Sender`
    ///
    /// `usart` is the USART transmitter that is used to send messages to the
    /// host.
    ///
    /// `buf` is the internal buffer used for serializing messages into.
    pub fn new(usart: &'r mut usart::Tx<USART>, buf: Buf) -> Self {
        Self {
            usart,
            buf,
        }
    }

    /// Send a message to the host
    pub fn send<T>(&mut self, message: &T) -> Result
        where T: Serialize
    {
        let serialized = postcard::to_slice_cobs(message, self.buf.as_mut())?;
        self.usart.bwrite_all(serialized)
            .void_unwrap();
        Ok(())
    }

}
