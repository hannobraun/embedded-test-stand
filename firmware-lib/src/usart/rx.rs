//! Receiving part of the interrupt-enabled USART API


use heapless::{
    Vec,
    spsc,
};
use lpc8xx_hal::{
    prelude::*,
    usart::{
        self,
        state::Enabled,
    },
};
use serde::Deserialize;

use super::QueueCap;


/// API for receiving data from a USART instance in an interrupt handler
///
/// You can get an instance of this struct by calling [`Usart::init`].
///
/// [`Usart::init`]: ../struct.Usart.html#method.init
pub struct RxInt<'r, I> {
    pub usart: usart::Rx<I, Enabled<u8>>,
    pub queue: spsc::Producer<'r, u8, QueueCap>,
}

impl<I> RxInt<'_, I>
    where
        I: usart::Instance,
{
    /// Receive available data
    ///
    /// This method should be called directly from the interrupt handler.
    /// Receives any available data, putting it into the internal queue, so it
    /// can be processed by the corresponging [`RxIdle`] instance without any
    /// time pressure.
    ///
    /// [`RxIdle`]: struct.RxIdle.html
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
/// You can get an instance of this struct by calling [`Usart::init`].
///
/// [`Usart::init`]: ../struct.Usart.html#method.init
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
    /// [`clear_buf`]. Otherwise, the same message will be processed again on
    /// the next call.
    ///
    /// [`clear_buf`]: #method.clear_buf
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
    /// This method _must_ be called after every call to [`process_message`], or
    /// on the next call, the same message will be processed again.
    ///
    /// It would be much nice, if this functionality could be included in
    /// [`process_message`], but unfortunately there's no straight-forward way
    /// to do this, as the lifetime required by the use of `Deserialize`
    /// interferes.
    ///
    /// [`process_message`]: #method.process_message
    pub fn clear_buf(&mut self) {
        self.buf.clear();
    }
}


/// Error receiving data from USART
#[derive(Debug)]
pub enum ReceiveError {
    /// The internal queue is full
    QueueFull,

    /// An error was returned by the wrapped USART receiver
    Usart(usart::Error),
}

/// Error processing received USART data
#[derive(Debug)]
pub enum ProcessError<E> {
    /// The internal buffer is full
    BufferFull,

    /// Error decoding the message
    Postcard(postcard::Error),

    /// Another error occured
    ///
    /// This is an error that was returned from the user-provided closure.
    Other(E),
}
