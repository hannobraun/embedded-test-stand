//! Library to support the test suite running on the host computer


pub mod config;
pub mod error;
pub mod receive;
pub mod serial;
pub mod send;
pub mod target;


pub use self::{
    error::{
        Error,
        Result,
    },
    receive::receive,
    send::send,
    target::Target,
};
