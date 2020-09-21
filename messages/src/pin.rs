//! Generic protocol related to pins
//!
//! The types in this module are not specific to any test stand setup, and can
//! be re-used for different test stands.


use serde::{
    Deserialize,
    Serialize,
};


/// Represents the electrical level of a pin
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum Level {
    High,
    Low,
}
