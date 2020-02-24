use std::{
    io,
    slice,
};

use serde::Deserialize;

use crate::Result;




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
