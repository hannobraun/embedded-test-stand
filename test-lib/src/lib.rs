#![cfg_attr(not(feature = "host"), no_std)]


#[cfg(feature = "firmware")]
mod firmware;

#[cfg(feature = "firmware")]
pub use firmware::*;


#[cfg(feature = "host")]
use std::{
    io,
    slice,
};

#[cfg(feature = "firmware")]
use lpc8xx_hal::{
    prelude::*,
    usart,
};
#[cfg(feature = "firmware")]
use void::ResultVoidExt;
use serde::{
    Deserialize,
    Serialize,
};


/// A request sent from the test suite to the firmware on the target
///
/// You can use [`Receiver`], to receive a request on the test target.
#[derive(Deserialize, Serialize)]
pub enum Request<'r> {
    /// Instruct the device to send a message via USART
    SendUsart(&'r [u8]),
}

impl<'r> Request<'r> {
    /// Send a request to the target, via the provided writer
    ///
    /// - `writer` is where the serialized request is written to.
    /// - `buf` is a buffer used for serialization. It needs to be big enough to
    ///   hold the serialized form of the request.
    ///
    /// This method is only available, if the `host` feature is enabled.
    #[cfg(feature = "host")]
    pub fn send<W: io::Write>(&self, mut writer: W, buf: &mut [u8]) -> Result {
        let serialized = postcard::to_slice_cobs(self, buf)?;
        writer.write_all(serialized)?;
        Ok(())
    }
}


/// An event that occured on the target
#[derive(Deserialize, Serialize)]
pub enum Event<'r> {
    UsartReceive(&'r [u8]),
}

impl<'r> Event<'r> {
    /// Send an event to the host, via the provided USART
    ///
    /// - `usart` is a USART instance that will be used to send this event.
    /// - `buf` is a buffer used for serialization. It needs to be large enough
    ///   to hold the serialized form of this event.
    ///
    /// This method is only available, if the `lpc8xx-hal` feature is enabled.
    #[cfg(feature = "firmware")]
    pub fn send<I>(&self, usart: &mut usart::Tx<I>, buf: &mut [u8]) -> Result
        where I: usart::Instance
    {
        let serialized = postcard::to_slice_cobs(self, buf)?;
        usart.bwrite_all(serialized)
            .void_unwrap();
        Ok(())
    }

    /// Receive a request from the target, via the provided reader
    ///
    /// - `reader` will be used to receive the request.
    /// - `buf` is a buffer that the request is read into, before it is
    ///   deserialized.
    ///
    /// This method is only available, if the `lpc8xx-hal` feature is enabled.
    #[cfg(feature = "host")]
    pub fn receive<R: io::Read>(mut reader: R, buf: &'r mut Vec<u8>)
        -> Result<Self>
    {
        loop {
            let mut b = 0; // initialized to `0`, but could be any value
            reader.read_exact(slice::from_mut(&mut b))?;

            buf.push(b);

            if b == 0 {
                // We're using COBS encoding, so `0` signifies the end of the
                // message.
                break;
            }
        }

        let event = postcard::from_bytes_cobs(buf)?;
        Ok(event)
    }
}


pub type Result<T = ()> = core::result::Result<T, Error>;


#[derive(Debug)]
pub enum Error {
    /// An I/O error occured
    ///
    /// This error is only available, if the `host` feature is enabled.
    #[cfg(feature = "host")]
    Io(io::Error),

    /// An error occured while using USART
    ///
    /// This error is only available, if the `firmware` feature is enabled.
    #[cfg(feature = "firmware")]
    Usart(usart::Error),

    /// An error originated from Postcard
    ///
    /// The `postcard` crate is used for (de-)serialization.
    Postcard(postcard::Error),

    /// The receive buffer is too small to receive a message
    BufferTooSmall,
}

#[cfg(feature = "host")]
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

#[cfg(feature = "firmware")]
impl From<usart::Error> for Error {
    fn from(err: usart::Error) -> Self {
        Self::Usart(err)
    }
}

impl From<postcard::Error> for Error {
    fn from(err: postcard::Error) -> Self {
        Self::Postcard(err)
    }
}
