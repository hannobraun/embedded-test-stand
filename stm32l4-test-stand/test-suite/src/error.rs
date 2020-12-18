/// Test-suite specific error module


use host_lib::assistant::AssistantError;

use crate::{
    target::{
        ReadAdcError,
        TargetI2cError,
        TargetPinReadError,
        TargetSetPinHighError,
        TargetSetPinLowError,
        TargetSpiError,
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
    Assistant(AssistantError),
    ReadAdc(ReadAdcError),
    TargetI2c(TargetI2cError),
    TargetPinRead(TargetPinReadError),
    TargetSetPinHigh(TargetSetPinHighError),
    TargetSetPinLow(TargetSetPinLowError),
    TargetSpi(TargetSpiError),
    TargetUsartSend(TargetUsartSendError),
    TargetUsartWait(TargetUsartWaitError),
    TestStandInit(TestStandInitError),
}

impl From<AssistantError> for Error {
    fn from(err: AssistantError) -> Self {
        Self::Assistant(err)
    }
}

impl From<ReadAdcError> for Error {
    fn from(err: ReadAdcError) -> Self {
        Self::ReadAdc(err)
    }
}

impl From<TargetI2cError> for Error {
    fn from(err: TargetI2cError) -> Self {
        Self::TargetI2c(err)
    }
}

impl From<TargetPinReadError> for Error {
    fn from(err: TargetPinReadError) -> Self {
        Self::TargetPinRead(err)
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

impl From<TargetSpiError> for Error {
    fn from(err: TargetSpiError) -> Self {
        Self::TargetSpi(err)
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
