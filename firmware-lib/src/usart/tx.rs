use lpc8xx_hal::usart;


/// Wraps a USART transmitter
///
/// Provides some convenience methods on top of the wrapped transmitter.
pub struct Tx<I> {
    pub usart: usart::Tx<I>,
}
