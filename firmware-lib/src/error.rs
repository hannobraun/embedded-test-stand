use lpc8xx_hal::usart;


pub type Result<T = ()> = core::result::Result<T, Error>;


#[derive(Debug)]
pub enum Error {
    /// An error occured while using USART
    ///
    /// This error is only available, if the `firmware` feature is enabled.
    Usart(usart::Error),

    /// An error originated from Postcard
    ///
    /// The `postcard` crate is used for (de-)serialization.
    Postcard(postcard::Error),
}

impl From<usart::Error> for Error {
    fn from(err: usart::Error) -> Self {
        Self::Usart(err)
    }
}

impl From<postcard::Error> for Error {
    fn from(err: postcard::Error) -> Self {
        Self::Postcard(err)
    }
}
