#![no_std]


use serde::{
    Deserialize,
    Serialize,
};


/// A message from the test suite on the host to the target
///
/// You can use [`Receiver`], to receive a message on the test target.
#[derive(Deserialize, Serialize)]
pub enum HostToTarget<'r> {
    /// Instruct the device to send a message via USART
    SendUsart(&'r [u8]),
}


/// An message from the target to the test suite on the host
#[derive(Deserialize, Serialize)]
pub enum TargetToHost<'r> {
    UsartReceive(&'r [u8]),
}
