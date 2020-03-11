use heapless::{
    Vec,
    consts::U256,
    spsc,
};
use lpc8xx_hal::{
    prelude::*,
    usart,
};


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


// It would be nice to make the queue capacity configurable, but that would
// require a generic with trait bound on all the structs. As of this writing,
// `const fn`s with trait bounds are unstable, so we can't do it yet.
type QueueCap = U256;


#[derive(Debug)]
pub enum ReceiveError {
    QueueFull,
    Usart(usart::Error),
}
