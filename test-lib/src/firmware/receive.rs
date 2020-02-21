use heapless::{
    ArrayLength,
    spsc,
};
use serde::Deserialize;

use crate::Error;


/// Receives and decodes host requests
pub struct Receiver<'a, QueueCap: ArrayLength<u8>, Buf: AsMut<[u8]>> {
    queue: &'a mut spsc::Consumer<'static, u8, QueueCap>,
    buf:   Buf,
    i:     usize,
}

impl<'a, QueueCap, Buf> Receiver<'a, QueueCap, Buf>
    where
        QueueCap: ArrayLength<u8>,
        Buf:      AsMut<[u8]>,
{
    /// Create a new instance of `Receiver`
    ///
    /// The `queue` argument is the queue consumer that receives bytes from the
    /// request.
    pub fn new(queue: &'a mut spsc::Consumer<'static, u8, QueueCap>, buf: Buf)
        -> Self
    {
        Self {
            queue,
            buf,
            i:   0,
        }
    }

    /// Indicates whether data can be received from the internal queue
    pub fn can_receive(&self) -> bool {
        self.queue.ready()
    }

    /// Receive bytes from the internal queue, return request if received
    ///
    /// This non-blocking method will receive bytes from the internal queue
    /// while they are available. If this leads to a full request being
    /// received, it will decode and return it.
    ///
    /// Returns `None`, if no full request has been received.
    pub fn receive<'de, T>(&'de mut self) -> Option<Result<T, Error>>
        where T: Deserialize<'de>
    {
        while let Some(b) = self.queue.dequeue() {
            self.buf.as_mut()[self.i] = b;
            self.i += 1;

            // Requests are COBS-encoded, so we know that `0` means we
            // received a full frame.
            if b == 0 {
                let data = postcard::from_bytes_cobs(self.buf.as_mut())
                    .map_err(|err| Error::Postcard(err));
                return Some(data);
            }
        }

        None
    }

    /// Reset the internal buffer
    ///
    /// This must be called each time a call to `receive` has returned `Some`.
    pub fn reset(&mut self) {
        self.i = 0;
    }
}
