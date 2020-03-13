use lpc8xx_hal::{
    prelude::*,
    usart,
};
use void::Void;


/// Wraps a USART transmitter
///
/// Provides some convenience methods on top of the wrapped transmitter.
pub struct Tx<I> {
    pub usart: usart::Tx<I>,
}

impl<I> Tx<I>
    where I: usart::Instance
{
    /// Sends raw data through the wrapped USART instance
    ///
    /// Blocks until the data has been sent.
    pub fn send_raw(&mut self, data: &[u8]) -> Result<(), Void> {
        self.usart.bwrite_all(data)
    }
}
