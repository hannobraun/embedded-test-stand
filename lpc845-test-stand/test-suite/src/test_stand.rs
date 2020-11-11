use std::sync::{
    LockResult,
    MutexGuard,
};

use host_lib::{
    assistant::Assistant,
    test_stand::NotConfiguredError,
};

use super::target::Target;


/// An instance of the test stand
///
/// Used to access all resources that a test case requires.
pub struct TestStand {
    _guard: LockResult<MutexGuard<'static, ()>>,

    pub target:    Target,
    pub assistant: Assistant,
}

impl TestStand {
    /// Initializes the test stand
    ///
    /// Reads the `test-stand.toml` configuration file and initializes test
    /// stand resources, as configured in there.
    pub fn new() -> Result<Self, TestStandInitError> {
        let test_stand = host_lib::TestStand::new()
            .map_err(|err| TestStandInitError::Inner(err))?;

        Ok(
            Self {
                _guard:    test_stand.guard,
                target:    Target::new(test_stand.target?),
                assistant: test_stand.assistant?,
            }
        )
    }
}


#[derive(Debug)]
pub enum TestStandInitError {
    Inner(host_lib::test_stand::TestStandInitError),
    NotConfigured(NotConfiguredError),
}

impl From<NotConfiguredError> for TestStandInitError {
    fn from(err: NotConfiguredError) -> Self {
        Self::NotConfigured(err)
    }
}
