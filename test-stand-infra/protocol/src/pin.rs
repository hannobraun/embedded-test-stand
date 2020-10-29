//! Generic protocol related to pins
//!
//! The types in this module are not specific to any test stand setup, and can
//! be re-used for different test stands.


use serde::{
    Deserialize,
    Serialize,
};


/// Sent by the host to command a test node to set a pin to a specific level
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct SetLevel<Id> {
    /// The pin whose level should be set
    pub pin: Id,

    /// The new level of the pin
    pub level: Level,
}


/// Sent by the host to request the current level of a pin
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct ReadLevel<Id> {
    /// The pin whose level to read
    pub pin: Id,
}


/// Sent by a test node in response to a `ReadLevel` message
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct ReadLevelResult<Id> {
    /// The pin whose level has changed
    pub pin: Id,

    /// The new level of the pin
    pub level: Level,

    /// The period since the last change of this pin's level, in milliseconds
    ///
    /// This value might not be available, because this is the first change of
    /// this pin's level, or because the test node doesn't measure the period.
    ///
    /// If the time since the last change has been too long, this value will
    /// not be reliable.
    pub period_ms: Option<u32>,
}


/// Represents the electrical level of a pin
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum Level {
    High,
    Low,
}
