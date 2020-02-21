#![cfg_attr(not(feature = "host"), no_std)]


#[cfg(feature = "firmware")]
mod firmware;
#[cfg(feature = "host")]
mod host;

#[cfg(feature = "firmware")]
pub use firmware::*;
#[cfg(feature = "host")]
pub use host::*;


#[cfg(feature = "host")]
use std::io;

#[cfg(feature = "firmware")]
use lpc8xx_hal::usart;
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


/// An event that occured on the target
#[derive(Deserialize, Serialize)]
pub enum Event<'r> {
    UsartReceive(&'r [u8]),
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
