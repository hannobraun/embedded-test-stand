//! Library to support the test suite running on the host computer


pub mod config;
pub mod error;
pub mod serial;
pub mod target;


pub use self::{
    error::{
        Error,
        Result,
    },
    target::Target,
};
