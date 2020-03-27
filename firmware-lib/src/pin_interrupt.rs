use heapless::{
    consts::U16,
    spsc::{
        Consumer,
        Producer,
        Queue,
    },
};
use lpc8xx_hal::{
    gpio,
    init_state::Enabled,
    pinint,
    pins,
};


/// Represents a pin interrupt
pub struct PinInterrupt {
    queue: Queue<gpio::Level, QueueCap>,
}

impl PinInterrupt {
    /// Create a new instance of `PinInterrupt`
    pub const fn new() -> Self {
        Self {
            queue: Queue(heapless::i::Queue::new()),
        }
    }

    /// Initialize pin interrupt
    ///
    /// Two new structs are returned:
    /// - [`Int`] is intended to be used from the interrupt context.
    /// - [`Idle`] is intended to be used from a lower-piority context, for
    ///   example the idle loop, to process events from the interrupt context.
    pub fn init<I, P>(&mut self, interrupt: pinint::Interrupt<I, P, Enabled>)
        -> (Int<I, P>, Idle)
    {
        let (prod, cons) = self.queue.split();

        let int  = Int { int: interrupt, queue: prod };
        let idle = Idle { queue: cons };

        (int, idle)
    }
}


/// Pin interrupt API for the interrupt context
pub struct Int<'r, I, P> {
    int:   pinint::Interrupt<I, P, Enabled>,
    queue: Producer<'r, gpio::Level, QueueCap>
}

impl<I, P> Int<'_, I, P>
    where
        I: pinint::Trait,
        P: pins::Trait,
{
    /// Handles a pin interrupts
    ///
    /// This should be called directly from the interrupt handler. Will check
    /// whether this interrupt was triggered by a rising or falling edge, and
    /// will send the respective event to the corresponding [`Idle`] instance.
    pub fn handle_interrupt(&mut self) {
        if self.int.clear_rising_edge_flag() {
            self.queue.enqueue(gpio::Level::High).unwrap();
        }
        if self.int.clear_falling_edge_flag() {
            self.queue.enqueue(gpio::Level::Low).unwrap();
        }
    }
}


/// Pin interrupt API for a lower-priority context
pub struct Idle<'r> {
    queue: Consumer<'r, gpio::Level, QueueCap>,
}

impl Idle<'_> {
    /// Returns the next pin interrupt event, if available
    pub fn next(&mut self) -> Option<gpio::Level> {
        self.queue.dequeue()
    }

    /// Indicates whether another pin interrupt event is available
    pub fn is_ready(&self) -> bool {
        self.queue.ready()
    }
}


// It would be nice to make the queue capacity configurable, but that would
// require a generic with trait bound on all the structs. As of this writing,
// `const fn`s with trait bounds are unstable, so we can't do it yet.
type QueueCap = U16;
