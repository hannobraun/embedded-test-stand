#![no_std]


pub mod pin;


use core::convert::TryFrom;

use serde::{
    Deserialize,
    Serialize,
};


/// A message from the test suite on the host to the test assistant
#[derive(Debug, Deserialize, Serialize)]
pub enum HostToAssistant<'r> {
    /// Instruct the assistant to send data to the target via USART
    SendUsart {
        mode: UsartMode,
        data: &'r [u8],
    },

    /// Instruct the assistant to change level of the target's input pin
    SetPin(pin::SetLevel<OutputPin>),

    /// Ask the assistant for the current level of a pin
    ReadPin(pin::ReadLevel<InputPin>),
}

impl From<pin::SetLevel<OutputPin>> for HostToAssistant<'_> {
    fn from(set_level: pin::SetLevel<OutputPin>) -> Self {
        Self::SetPin(set_level)
    }
}

impl From<pin::ReadLevel<InputPin>> for HostToAssistant<'_> {
    fn from(read_level: pin::ReadLevel<InputPin>) -> Self {
        Self::ReadPin(read_level)
    }
}


/// A message from the test assistant to the test suite on the host
#[derive(Debug, Deserialize, Serialize)]
pub enum AssistantToHost<'r> {
    /// Notify the host that data has been received from the target via USART
    UsartReceive {
        mode: UsartMode,
        data: &'r [u8],
    },

    /// Notify the host that the level of a pin has changed
    ReadPinResult(Option<pin::ReadLevelResult<InputPin>>),
}

impl<'r> TryFrom<AssistantToHost<'r>> for pin::ReadLevelResult<InputPin> {
    type Error = AssistantToHost<'r>;

    fn try_from(value: AssistantToHost<'r>) -> Result<Self, Self::Error> {
        match value {
            AssistantToHost::ReadPinResult(Some(result)) => {
                Ok(result)
            }
            _ => {
                Err(value)
            }
        }
    }
}


/// Specifies which mode a USART transmission uses
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum UsartMode {
    Regular,
    Dma,
    FlowControl,
    Sync,
}


/// Represents one of the pins that the assistant is monitoring
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum InputPin {
    Blue  = 0,
    Green = 1,
    Rts   = 2,
}

/// Represents one of the pins that the assistant can set
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum OutputPin {
    Pin5,
    Cts,
    Red,
}
