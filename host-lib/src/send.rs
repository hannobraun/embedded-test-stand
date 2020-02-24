use std::io;

use serde::Serialize;

use crate::Result;


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
