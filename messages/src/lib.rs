#![no_std]


use serde::{
    Deserialize,
    Serialize,
};


/// A message from the test suite on the host to the target
#[derive(Debug, Deserialize, Serialize)]
pub enum HostToTarget<'r> {
    /// Instruct the target to send a message via USART
    SendUsart(Mode, &'r [u8]),

    /// Instruct the device to change the electrical level of the pin
    SetPin(PinState),

    /// Instruct the target to start the timer interrupt
    StartTimerInterrupt { period_ms: u32 },

    /// Instruct the target to stop the timer interrupt
    StopTimerInterrupt,

    /// Instruct the target to start an I2C transaction
    StartI2cTransaction {
        /// Which mode to use for the transaction
        mode: Mode,

        /// The address of the slave
        address: u8,

        /// The data to send to the slave
        data: u8,
    },

    /// Instruct the target to start an SPI transaction
    StartSpiTransaction {
        /// Which mode to use for the transaction
        mode: Mode,

        /// The data to send to the slave
        data: u8,
    },
}

/// An message from the target to the test suite on the host
#[derive(Debug, Deserialize, Serialize)]
pub enum TargetToHost<'r> {
    /// Notify the host that data has been received via USART
    UsartReceive(Mode, &'r [u8]),

    /// Notify the host that the level of GPIO input changed
    PinLevelChanged {
        /// The new level of the pin
        level: PinState,
    },

    /// Notify the host that the I2C transaction completed
    I2cReply(u8),

    /// Notify the host that the SPI transaction completed
    SpiReply(u8),
}


/// A message from the test suite on the host to the test assistant
#[derive(Debug, Deserialize, Serialize)]
pub enum HostToAssistant<'r> {
    /// Instruct the assistant to send data to the target via USART
    SendUsart(Mode, &'r [u8]),

    /// Instruct the assistant to change level of the target's input pin
    SetPin(PinState),
}

/// A message from the test assistant to the test suite on the host
#[derive(Debug, Deserialize, Serialize)]
pub enum AssistantToHost<'r> {
    /// Notify the host that data has been received from the target via USART
    UsartReceive(&'r [u8]),

    /// Notify the host that the level of a pin has changed
    PinLevelChanged {
        /// The pin whose level has changed
        pin: InputPin,

        /// The new level of the pin
        level: PinState,

        /// The period since the last change of this pin in ms, if available
        ///
        /// If the time since the last change has been too long, this value will
        /// not be reliable.
        period_ms: Option<u32>,
    }
}


/// Specifies whether a USART transmission concerns DMA or not
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum Mode {
    Regular,
    Dma,
}


/// Represents one of the pins that the assistant is monitoring
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum InputPin {
    Blue,
    Green,
    Rts,
}


/// Represents the electrical level of a pin
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum PinState {
    High,
    Low,
}
