use heapless::{
    Vec,
    consts::U256,
    spsc,
};
use lpc8xx_hal::{
    prelude::*,
    usart,
};
use serde::Deserialize;


/// Interrupt-enabled USART receiver
///
/// Can be allocated in a `static` or another memory location with an
/// appropriate lifetime. Once initialized, it is split into two parts:
///
/// - [`RxInt`], which handles the timing-critical parts of receiving, and is
///   intended to be moved into the interrupt handler.
/// - [`RxIdle`], which can be used to process the received data.
pub struct Rx {
    queue: spsc::Queue<u8, QueueCap>,
}

impl Rx {
    /// Create a new instance of `Rx`
    pub const fn new() -> Self {
        Self {
            queue: spsc::Queue(heapless::i::Queue::new()),
        }
    }

    /// Initialize the receiver
    ///
    /// Returns the two parts, [`RxInt`] and [`RxIdle`], which can then be moved
    /// into different contexts.
    pub fn init<I>(&mut self, usart: usart::Rx<I>) -> (RxInt<I>, RxIdle) {
        let (prod, cons) = self.queue.split();

        let rx_int = RxInt {
            usart,
            queue: prod,
        };
        let rx_idle = RxIdle {
            queue: cons,
            buf:   Vec::new(),
        };

        (rx_int, rx_idle)
    }
}


/// API for receiving data from a USART instance in an interrupt handler
///
/// You can get an instance of this struct from [`Rx::init`].
pub struct RxInt<'r, I> {
    pub usart: usart::Rx<I>,
    pub queue: spsc::Producer<'r, u8, QueueCap>,
}

impl<I> RxInt<'_, I>
    where
        I: usart::Instance,
{
    pub fn receive(&mut self) -> Result<(), ReceiveError> {
        loop {
            match self.usart.read() {
                Ok(b) => {
                    self.queue.enqueue(b)
                        .map_err(|_| ReceiveError::QueueFull)?;
                }
                Err(nb::Error::WouldBlock) => {
                    return Ok(());
                }
                Err(nb::Error::Other(err)) => {
                    return Err(ReceiveError::Usart(err));
                }
            }
        }
    }
}


/// API for processing received data
///
/// This processing can be done in a lower-priority context, for example an idle
/// loop.
///
/// You can get an instance of this struct from [`Rx::init`].
pub struct RxIdle<'r> {
    pub queue: spsc::Consumer<'r, u8, QueueCap>,
    pub buf:   Vec<u8, QueueCap>,
}

impl RxIdle<'_> {
    /// Indicates whether data has been received that can be processed
    pub fn can_process(&self) -> bool {
        self.queue.ready()
    }

    /// Process received data
    ///
    /// Copies any available data to the internal buffer. If the buffer is not
    /// empty, the closure is called, with the buffer data as an argument.
    ///
    /// The internal buffer is cleared, once the closure returns.
    pub fn process_raw<E>(&mut self, f: impl FnOnce(&[u8]) -> Result<(), E>)
        -> Result<(), ProcessError<E>>
    {
        while let Some(b) = self.queue.dequeue() {
            self.buf.push(b)
                .map_err(|_| ProcessError::BufferFull)?;
        }

        if self.buf.len() > 0 {
            f(&self.buf)
                .map_err(|err| ProcessError::Other(err))?;
            self.buf.clear();
        }

        Ok(())
    }

    /// Process received message
    ///
    /// Copies any available data to the internal buffer until no more data is
    /// available, or a full message has been received. If a message has been
    /// received, that message is deserialized and the closure is called.
    ///
    /// After calling this method, you must clear the internal buffer by calling
    /// [`RxIdle::clear_buf`]. Otherwise, the same message will be processed
    /// again on the next call.
    pub fn process_message<'de, M, E>(&'de mut self,
        f: impl FnOnce(M) -> Result<(), E>,
    )
        -> Result<(), ProcessError<E>>
        where M: Deserialize<'de>
    {
        while let Some(b) = self.queue.dequeue() {
            self.buf.push(b)
                .map_err(|_| ProcessError::BufferFull)?;

            // Requests are COBS-encoded, so we know that `0` means we
            // received a full frame.
            if b == 0 {
                let message = postcard::from_bytes_cobs(&mut self.buf)
                    .map_err(|err| ProcessError::Postcard(err))?;
                f(message)
                    .map_err(|err| ProcessError::Other(err))?;
                return Ok(());
            }
        }

        Ok(())
    }

    /// Clear the internal buffer
    ///
    /// This method _must_ be called after every call to `process_message`, or
    /// on the next call, the same message will be processed again.
    ///
    /// It would be much nice, if this functionality could be included in
    /// `process_message`, but unfortunately there's no straight-forward way to
    /// do this, as the lifetime required by the use of `Deserialize`
    /// interferes.
    pub fn clear_buf(&mut self) {
        self.buf.clear();
    }
}


// It would be nice to make the queue capacity configurable, but that would
// require a generic with trait bound on all the structs. As of this writing,
// `const fn`s with trait bounds are unstable, so we can't do it yet.
type QueueCap = U256;


#[derive(Debug)]
pub enum ReceiveError {
    QueueFull,
    Usart(usart::Error),
}

#[derive(Debug)]
pub enum ProcessError<E> {
    BufferFull,
    Postcard(postcard::Error),
    Other(E),
}
