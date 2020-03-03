#![no_std]


use serde::{
    Deserialize,
    Serialize,
};


/// A message from the test suite on the host to the target
#[derive(Deserialize, Serialize)]
pub enum HostToTarget<'r> {
    /// Instruct the target to send a message via USART
    SendUsart(&'r [u8]),

    /// Instruct the device to set a specific pin high
    SetPinHigh,

    /// Instruct the target to set a specific pin low
    SetPinLow,
}

/// An message from the target to the test suite on the host
#[derive(Deserialize, Serialize)]
pub enum TargetToHost<'r> {
    /// Notify the host that data has been received via USART
    UsartReceive(&'r [u8]),
}


/// A message from the test suite on the host to the test assistant
#[derive(Deserialize, Serialize)]
pub enum HostToAssistant<'r> {
    /// Instruct the assistant to send data to the target via USART
    SendUsart(&'r [u8]),
}

/// A message from the test assistant to the test suite on the host
#[derive(Deserialize, Serialize)]
pub enum AssistantToHost<'r> {
    /// Notify the host that data has been received from the target via USART
    UsartReceive(&'r [u8]),
}
