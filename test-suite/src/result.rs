use super::{
    serial::{
        SerialSendError,
        SerialWaitError,
    },
    target::{
        TargetSendError,
        TargetUsartWaitError,
    },
    test_stand::TestStandInitError,
};


/// Result type specific to this test suite
pub type Result<T = ()> = std::result::Result<T, Error>;


/// Error type specific to this test suite
#[derive(Debug)]
pub enum Error {
    SerialSend(SerialSendError),
    SerialWait(SerialWaitError),
    TargetSend(TargetSendError),
    TargetUsartWait(TargetUsartWaitError),
    TestStandInit(TestStandInitError),
}

impl From<SerialSendError> for Error {
    fn from(err: SerialSendError) -> Self {
        Self::SerialSend(err)
    }
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

impl From<TargetUsartWaitError> for Error {
    fn from(err: TargetUsartWaitError) -> Self {
        Self::TargetUsartWait(err)
    }
}

impl From<TestStandInitError> for Error {
    fn from(err: TestStandInitError) -> Self {
        Self::TestStandInit(err)
    }
}
