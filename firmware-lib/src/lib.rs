#![no_std]


pub mod error;
pub mod receive;
pub mod send;


pub use self::{
    error::{
        Error,
        Result,
    },
    receive::Receiver,
    send::Sender,
};
