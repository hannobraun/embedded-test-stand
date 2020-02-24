use std::io;


pub type Result<T = ()> = core::result::Result<T, Error>;


/// Various low-level errors that can occur in the test suite support code
#[derive(Debug)]
pub enum Error {
    Config(toml::de::Error),

    /// An I/O error occured
    Io(io::Error),

    /// An error originated from Postcard
    ///
    /// The `postcard` crate is used for (de-)serialization.
    Postcard(postcard::Error),

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
