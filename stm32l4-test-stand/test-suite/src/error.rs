/// Test-suite specific error module


use host_lib::assistant::AssistantError;

use crate::{
    target::TargetUsartSendError,
    test_stand::TestStandInitError,
};


/// Result type specific to this test suite
pub type Result<T = ()> = std::result::Result<T, Error>;


/// Error type specific to this test suite
#[derive(Debug)]
pub enum Error {
    Assistant(AssistantError),
    TargetUsartSend(TargetUsartSendError),
    TestStandInit(TestStandInitError),
}

impl From<AssistantError> for Error {
    fn from(err: AssistantError) -> Self {
        Self::Assistant(err)
    }
}

impl From<TargetUsartSendError> for Error {
    fn from(err: TargetUsartSendError) -> Self {
        Self::TargetUsartSend(err)
    }
}

impl From<TestStandInitError> for Error {
    fn from(err: TestStandInitError) -> Self {
        Self::TestStandInit(err)
    }
}
