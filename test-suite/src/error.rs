/// Test-suite specific error module


use host_lib::{
    serial::{
        SerialSendError,
        SerialWaitError,
    },
    test_stand::NotConfiguredError,
};

use super::{
    assistant::{
        AssistantPinReadError,
        AssistantSetPinHighError,
        AssistantSetPinLowError,
        AssistantUsartSendError,
        AssistantUsartWaitError,
    },
    target::{
        TargetPinReadError,
        TargetSetPinHighError,
        TargetSetPinLowError,
        TargetStartTimerInterruptError,
        TargetUsartSendError,
        TargetUsartWaitError,
    },
    test_stand::TestStandInitError,
};


/// Result type specific to this test suite
pub type Result<T = ()> = std::result::Result<T, Error>;


/// Error type specific to this test suite
#[derive(Debug)]
pub enum Error {
    AssistantPinRead(AssistantPinReadError),
    AssistantSetPinHigh(AssistantSetPinHighError),
    AssistantSetPinLow(AssistantSetPinLowError),
    AssistantUsartSend(AssistantUsartSendError),
    AssistantUsartWait(AssistantUsartWaitError),
    NotConfigured(NotConfiguredError),
    SerialSend(SerialSendError),
    SerialWait(SerialWaitError),
    TargetPinRead(TargetPinReadError),
    TargetSetPinHigh(TargetSetPinHighError),
    TargetSetPinLow(TargetSetPinLowError),
    TargetStartTimerInterrupt(TargetStartTimerInterruptError),
    TargetUsartSend(TargetUsartSendError),
    TargetUsartWait(TargetUsartWaitError),
    TestStandInit(TestStandInitError),
}

impl From<AssistantPinReadError> for Error {
    fn from(err: AssistantPinReadError) -> Self {
        Self::AssistantPinRead(err)
    }
}

impl From<AssistantSetPinHighError> for Error {
    fn from(err: AssistantSetPinHighError) -> Self {
        Self::AssistantSetPinHigh(err)
    }
}

impl From<AssistantSetPinLowError> for Error {
    fn from(err: AssistantSetPinLowError) -> Self {
        Self::AssistantSetPinLow(err)
    }
}

impl From<AssistantUsartSendError> for Error {
    fn from(err: AssistantUsartSendError) -> Self {
        Self::AssistantUsartSend(err)
    }
}

impl From<AssistantUsartWaitError> for Error {
    fn from(err: AssistantUsartWaitError) -> Self {
        Self::AssistantUsartWait(err)
    }
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

impl From<TargetPinReadError> for Error {
    fn from(err: TargetPinReadError) -> Self {
        Self::TargetPinRead(err)
    }
}

impl From<TargetStartTimerInterruptError> for Error {
    fn from(err: TargetStartTimerInterruptError) -> Self {
        Self::TargetStartTimerInterrupt(err)
    }
}

impl From<TargetUsartSendError> for Error {
    fn from(err: TargetUsartSendError) -> Self {
        Self::TargetUsartSend(err)
    }
}

impl From<TargetSetPinHighError> for Error {
    fn from(err: TargetSetPinHighError) -> Self {
        Self::TargetSetPinHigh(err)
    }
}

impl From<TargetSetPinLowError> for Error {
    fn from(err: TargetSetPinLowError) -> Self {
        Self::TargetSetPinLow(err)
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
