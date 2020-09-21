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


/// Represents the electrical level of a pin
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum Level {
    High,
    Low,
}
