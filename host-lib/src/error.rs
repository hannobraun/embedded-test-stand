/// Defines the error type for this library


use std::io;


/// The result type for this library
///
/// This is just a convenient short-hand.
pub type Result<T = ()> = core::result::Result<T, Error>;


/// The error type for this library
#[derive(Debug)]
pub enum Error {
    /// Error occured while deserializing the configuration file
    Config(toml::de::Error),

    /// An I/O error occured
    Io(io::Error),

    /// An error originated from Postcard
    ///
    /// The `postcard` crate is used for (de-)serialization.
    Postcard(postcard::Error),

    /// Error occured while accessing the serial port
    Serial(serialport::Error),
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Self::Config(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<postcard::Error> for Error {
    fn from(err: postcard::Error) -> Self {
        Self::Postcard(err)
    }
}

impl From<serialport::Error> for Error {
    fn from(err: serialport::Error) -> Self {
        Self::Serial(err)
    }
}
