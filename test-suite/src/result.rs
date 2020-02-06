use std::io;

use super::{
    serial::SerialWaitError,
    target::TargetSendError,
    test_stand::TestStandInitError,
};


/// Result type specific to this test suite
pub type Result<T = ()> = std::result::Result<T, Error>;


/// Error type specific to this test suite
#[derive(Debug)]
pub enum Error {
    SerialWait(SerialWaitError),
    TargetSend(TargetSendError),
    TestStandInit(TestStandInitError),
}

impl From<SerialWaitError> for Error {
    fn from(err: SerialWaitError) -> Self {
        Self::SerialWait(err)
    }
}

impl From<TargetSendError> for Error {
    fn from(err: TargetSendError) -> Self {
        Self::TargetSend(err)
    }
}

impl From<TestStandInitError> for Error {
    fn from(err: TestStandInitError) -> Self {
        Self::TestStandInit(err)
    }
}


/// Various low-level errors that can occur in the test suite support code
#[derive(Debug)]
pub enum LowLevelError {
    Config(toml::de::Error),
    Io(io::Error),
    Serial(serialport::Error),
    TestLib(lpc845_test_lib::Error),
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

impl From<lpc845_test_lib::Error> for LowLevelError {
    fn from(err: lpc845_test_lib::Error) -> Self {
        Self::TestLib(err)
    }
}
