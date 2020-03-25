use std::sync::{
    LockResult,
    MutexGuard,
};

use host_lib::{
    conn::Conn,
    serial::Serial,
    test_stand::{
        NotConfiguredError,
        TestStandInitError,
    },
};

use super::{
    assistant::Assistant,
    target::Target,
};


/// An instance of the test stand
///
/// Used to access all resources that a test case requires.
pub struct TestStand {
    _guard: LockResult<MutexGuard<'static, ()>>,

    pub target:    Option<Conn>,
    pub assistant: Option<Conn>,
    pub serial:    Option<Serial>,
}

impl TestStand {
    /// Initializes the test stand
    ///
    /// Reads the `test-stand.toml` configuration file and initializes test
    /// stand resources, as configured in there.
    pub fn new() -> Result<Self, TestStandInitError> {
        let test_stand = host_lib::TestStand::new()?;

        Ok(
            Self {
                _guard:    test_stand.guard,
                target:    test_stand.target,
                assistant: test_stand.assistant,
                serial:    test_stand.serial,
            }
        )
    }

    /// Returns the connection to the test target (device under test)
    pub fn target(&mut self) -> Result<Target, NotConfiguredError> {
        match &mut self.target {
            Some(target) => Ok(Target(target)),
            None         => Err(NotConfiguredError("target")),
        }
    }

    /// Returns the connection to the test assistant
    pub fn assistant(&mut self) -> Result<Assistant, NotConfiguredError> {
        match &mut self.assistant {
            Some(assistant) => Ok(Assistant(assistant)),
            None            => Err(NotConfiguredError("assistant")),
        }
    }

    /// Returns the connection to the Serial-to-USB converter
    pub fn serial(&mut self) -> Result<&mut Serial, NotConfiguredError> {
        match &mut self.serial {
            Some(serial) => Ok(serial),
            None         => Err(NotConfiguredError("serial")),
        }
    }
}
