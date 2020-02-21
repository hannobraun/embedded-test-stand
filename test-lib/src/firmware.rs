use heapless::{
    ArrayLength,
    spsc,
};

use super::{
    Error,
    Request,
};


/// Receives and decodes host requests
pub struct Receiver<'a, Capacity: ArrayLength<u8>> {
    queue: &'a mut spsc::Consumer<'static, u8, Capacity>,
    buf:   [u8; 256],
    i:     usize,
}

impl<'a, Capacity> Receiver<'a, Capacity>
    where Capacity: ArrayLength<u8>
{
    /// Create a new instance of `Receiver`
    ///
    /// The `queue` argument is the queue consumer that receives bytes from the
    /// request.
    pub fn new(queue: &'a mut spsc::Consumer<'static, u8, Capacity>) -> Self {
        Self {
            queue,
            buf: [0; 256],
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
    pub fn receive(&mut self) -> Option<Result<Request, Error>> {
        while let Some(b) = self.queue.dequeue() {
            self.buf[self.i] = b;
            self.i += 1;

            // Requests are COBS-encoded, so we know that `0` means we
            // received a full frame.
            if b == 0 {
                return Some(Request::deserialize(&mut self.buf));
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
