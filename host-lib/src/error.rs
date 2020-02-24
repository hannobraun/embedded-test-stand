use std::io;


pub type Result<T = ()> = core::result::Result<T, Error>;


#[derive(Debug)]
pub enum Error {
    /// An I/O error occured
    Io(io::Error),

    /// An error originated from Postcard
    ///
    /// The `postcard` crate is used for (de-)serialization.
    Postcard(postcard::Error),
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


/// Various low-level errors that can occur in the test suite support code
#[derive(Debug)]
pub enum LowLevelError {
    Config(toml::de::Error),
    Io(io::Error),
    Serial(serialport::Error),
    TestLib(Error),
}

impl From<toml::de::Error> for LowLevelError {
    fn from(err: toml::de::Error) -> Self {
        Self::Config(err)
    }
}

impl From<io::Error> for LowLevelError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serialport::Error> for LowLevelError {
    fn from(err: serialport::Error) -> Self {
        Self::Serial(err)
    }
}

impl From<Error> for LowLevelError {
    fn from(err: Error) -> Self {
        Self::TestLib(err)
    }
}
