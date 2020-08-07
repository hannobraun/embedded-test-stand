//! Sending part of the interrupt-enabled USART API


use lpc8xx_hal::{
    prelude::*,
    usart::{
        self,
        state::{
            Enabled,
            NoThrottle,
        },
    },
};
use serde::Serialize;
use void::{
    ResultVoidExt,
    Void,
};


/// Wraps a USART transmitter
///
/// Provides some convenience methods on top of the wrapped transmitter.
pub struct Tx<I> {
    pub usart: usart::Tx<I, Enabled<u8>, NoThrottle>,
}

impl<I> Tx<I>
    where I: usart::Instance
{
    /// Sends raw data through the wrapped USART instance
    ///
    /// Blocks until the data has been sent.
    pub fn send_raw(&mut self, data: &[u8]) -> Result<(), Void> {
        self.usart.bwrite_all(data)
    }

    /// Sends a message through the wrapped USART instance
    ///
    /// Accepts a message and a buffer. The buffer will be used to hold the
    /// serialized message, and must be large enough for that purpose. Any
    /// previous contents of the buffer will be ignored.
    pub fn send_message<T>(&mut self, message: &T, buf: &mut [u8])
        -> Result<(), Error>
        where T: Serialize
    {
        let data = postcard::to_slice_cobs(message, buf)?;
        self.usart.bwrite_all(data)
            .void_unwrap();
        Ok(())
    }
}


/// Error occurred while serializing message
pub type Error = postcard::Error;
