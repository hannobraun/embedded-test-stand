//! Convenient pin interrupt API


use heapless::{
    consts::U32,
    spsc::{
        Consumer,
        Producer,
        Queue,
    },
};
use lpc8xx_hal::{
    prelude::*,
    gpio,
    init_state::Enabled,
    mrt,
    pinint,
    pins,
};


/// Represents a pin interrupt
pub struct PinInterrupt {
    queue: Queue<Event, QueueCap>,
}

impl PinInterrupt {
    /// Create a new instance of `PinInterrupt`
    ///
    /// Can be called in a const context, which means it can be used to
    /// initialize a `static`.
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
    ///
    /// Both structs have a lifetime that is tied to the lifetime of `self`.
    /// This can be prohibitive, if you're going to move the structs into
    /// different contexts. It is recommended to avoid this problem by
    /// allocating the `PinInterrupt` struct in a `static`.
    ///
    /// [`Int`]: struct.Int.html
    /// [`Idle`]: struct.Idle.html
    pub fn init<I, P, T: mrt::Trait>(&mut self,
        interrupt: pinint::Interrupt<I, P, Enabled>,
        timer:     mrt::Channel<T>,
    )
        -> (Int<I, P, T>, Idle)
    {
        let (prod, cons) = self.queue.split();

        let int  = Int { int: interrupt, queue: prod, timer, measuring: false };
        let idle = Idle { queue: cons };

        (int, idle)
    }
}


/// Pin interrupt API for the interrupt context
///
/// You can get an instance of this struct by calling [`PinInterrupt::init`].
/// The `Int` instance can then be moved into the interrupt handler.
///
/// [`PinInterrupt::init`]: struct.PinInterrupt.html#method.init
pub struct Int<'r, I, P, T: mrt::Trait> {
    int:       pinint::Interrupt<I, P, Enabled>,
    queue:     Producer<'r, Event, QueueCap>,
    timer:     mrt::Channel<T>,
    measuring: bool,
}

impl<I, P, T> Int<'_, I, P, T>
    where
        I: pinint::Trait,
        P: pins::Trait,
        T: mrt::Trait,
{
    /// Handles a pin interrupts
    ///
    /// This should be called directly from the interrupt handler. Will check
    /// whether this interrupt was triggered by a rising or falling edge, and
    /// will send the respective event to the corresponding [`Idle`] instance.
    ///
    /// [`Idle`]: struct.Idle.html
    pub fn handle_interrupt(&mut self) {
        let mut period = None;

        if self.measuring {
            let timer_wrapped = self.timer.wait().is_ok();
            if !timer_wrapped {
                period = Some(mrt::MAX_VALUE.to_u32() - self.timer.value());
            }
        }

        self.timer.start(mrt::MAX_VALUE);
        self.measuring = true;

        if self.int.clear_rising_edge_flag() {
            let event = Event { level: gpio::Level::High, period };
            self.queue.enqueue(event).unwrap();
        }
        if self.int.clear_falling_edge_flag() {
            let event = Event { level: gpio::Level::Low, period };
            self.queue.enqueue(event).unwrap();
        }
    }
}


/// Pin interrupt API for a lower-priority context
///
/// You can get an instance of this struct by calling [`PinInterrupt::init`].
/// The `Idle` instance can then be moved to a lower-priority context, for
/// example the idle loop, where it can be used to process events received from
/// the corresponding [`Int`] instance without any time pressure.
///
/// [`PinInterrupt::init`]: struct.PinInterrupt.html#method.init
/// [`Int`]: struct.Int.html
pub struct Idle<'r> {
    queue: Consumer<'r, Event, QueueCap>,
}

impl Idle<'_> {
    /// Returns the next pin interrupt event, if available
    pub fn next(&mut self) -> Option<Event> {
        self.queue.dequeue()
    }

    /// Indicates whether another pin interrupt event is available
    pub fn is_ready(&self) -> bool {
        self.queue.ready()
    }
}


/// A pin interrupt event
#[derive(Debug)]
pub struct Event {
    /// The level of the pin after this event
    pub level: gpio::Level,

    /// The period measured since the last event, if available
    pub period: Option<u32>,
}


// It would be nice to make the queue capacity configurable, but that would
// require a generic with trait bound on all the structs. As of this writing,
// `const fn`s with trait bounds are unstable, so we can't do it yet.
type QueueCap = U32;
