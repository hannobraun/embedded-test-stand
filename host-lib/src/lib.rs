use std::{
    io,
    slice,
};

use serde::{
    Deserialize,
    Serialize,
};


/// Send a message to the target, via the provided writer
///
/// - `writer` is where the serialized request is written to.
/// - `buf` is a buffer used for serialization. It needs to be big enough to
///   hold the serialized form of the request.
pub fn send<T, W>(message: &T, mut writer: W, buf: &mut [u8]) -> Result
    where
        T: Serialize,
        W: io::Write,
{
    let serialized = postcard::to_slice_cobs(message, buf)?;
    writer.write_all(serialized)?;
    Ok(())
}

/// Receive a message from the target, via the provided reader
///
/// - `reader` will be used to receive the request.
/// - `buf` is a buffer that the request is read into, before it is
///   deserialized.
pub fn receive<'de, T, R>(mut reader: R, buf: &'de mut Vec<u8>) -> Result<T>
    where
        T: Deserialize<'de>,
        R: io::Read,
{
    loop {
        let mut b = 0; // initialized to `0`, but could be any value
        reader.read_exact(slice::from_mut(&mut b))?;

        buf.push(b);

        if b == 0 {
            // We're using COBS encoding, so `0` signifies the end of the
            // message.
            break;
        }
    }

    let event = postcard::from_bytes_cobs(buf)?;
    Ok(event)
}


pub type Result<T = ()> = core::result::Result<T, Error>;


#[derive(Debug)]
pub enum Error {
    /// An I/O error occured
    Io(io::Error),

    /// An error originated from Postcard
    ///
    /// The `postcard` crate is used for (de-)serialization.
    Postcard(postcard::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<postcard::Error> for Error {
    fn from(err: postcard::Error) -> Self {
        Self::Postcard(err)
    }
}
