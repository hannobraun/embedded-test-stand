pub mod rx;
pub mod tx;


pub use self::{
    rx::{
        RxIdle,
        RxInt,
    },
    tx::Tx,
};


use heapless::{
    Vec,
    consts::U256,
    spsc,
};
use lpc8xx_hal::USART;


/// Interrupt-enabled USART wrapper
///
/// Can be allocated in a `static` or another memory location with an
/// appropriate lifetime. Once initialized, it is split into two parts:
///
/// - [`RxInt`], which handles the timing-critical parts of receiving, and is
///   intended to be moved into the interrupt handler.
/// - [`RxIdle`], which can be used to process the received data somewhere else.
/// - [`Tx`], which can be used to send data.
pub struct Usart {
    queue: spsc::Queue<u8, QueueCap>,
}

impl Usart {
    /// Creates a new instance of `Usart`
    pub const fn new() -> Self {
        Self {
            queue: spsc::Queue(heapless::i::Queue::new()),
        }
    }

    /// Initialize the USART
    ///
    /// Returns the three parts - [`RxInt`], [`RxIdle`], and [`Tx`] - which can
    /// then be moved into different contexts.
    pub fn init<I>(&mut self, usart: USART<I>) -> (RxInt<I>, RxIdle, Tx<I>) {
        let (prod, cons) = self.queue.split();

        let rx_int = RxInt {
            usart: usart.rx,
            queue: prod,
        };
        let rx_idle = RxIdle {
            queue: cons,
            buf:   Vec::new(),
        };
        let tx = Tx {
            usart: usart.tx,
        };

        (rx_int, rx_idle, tx)
    }
}


// It would be nice to make the queue capacity configurable, but that would
// require a generic with trait bound on all the structs. As of this writing,
// `const fn`s with trait bounds are unstable, so we can't do it yet.
type QueueCap = U256;
