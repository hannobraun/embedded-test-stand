use host_lib::{
    serial::{
        SerialSendError,
        SerialWaitError,
    },
    test_stand::TestStandInitError,
};

use super::{
    target::{
        TargetUsartSendError,
        TargetUsartWaitError,
    },
    test_stand::NotConfiguredError,
};


/// Result type specific to this test suite
pub type Result<T = ()> = std::result::Result<T, Error>;


/// Error type specific to this test suite
#[derive(Debug)]
pub enum Error {
    NotConfigured(NotConfiguredError),
    SerialSend(SerialSendError),
    SerialWait(SerialWaitError),
    TargetUsartSend(TargetUsartSendError),
    TargetUsartWait(TargetUsartWaitError),
    TestStandInit(TestStandInitError),
}

impl From<NotConfiguredError> for Error {
    fn from(err: NotConfiguredError) -> Self {
        Self::NotConfigured(err)
    }
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

impl From<TargetUsartSendError> for Error {
    fn from(err: TargetUsartSendError) -> Self {
        Self::TargetUsartSend(err)
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
