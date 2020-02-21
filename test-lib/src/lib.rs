#![cfg_attr(not(feature = "host"), no_std)]


#[cfg(feature = "firmware")]
mod firmware;
#[cfg(feature = "host")]
mod host;

#[cfg(feature = "firmware")]
pub use firmware::*;
#[cfg(feature = "host")]
pub use host::*;


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
