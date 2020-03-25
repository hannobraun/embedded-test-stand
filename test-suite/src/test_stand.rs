use std::sync::{
    LockResult,
    MutexGuard,
};

use host_lib::{
    conn::Conn,
    serial::Serial,
    test_stand::NotConfiguredError,
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

    pub target:    Result<Conn, NotConfiguredError>,
    pub assistant: Result<Conn, NotConfiguredError>,
    pub serial:    Result<Serial, NotConfiguredError>,
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
                target:    test_stand.target,
                assistant: test_stand.assistant,
                serial:    test_stand.serial,
            }
        )
    }

    /// Returns the connection to the test target (device under test)
    pub fn target(&mut self) -> Result<Target, NotConfiguredError> {
        self.target.as_mut()
            .map(|target| Target(target))
            .map_err(|err| *err)
    }

    /// Returns the connection to the test assistant
    pub fn assistant(&mut self) -> Result<Assistant, NotConfiguredError> {
        self.assistant.as_mut()
            .map(|assistant| Assistant(assistant))
            .map_err(|err| *err)
    }

    /// Returns the connection to the Serial-to-USB converter
    pub fn serial(&mut self) -> Result<&mut Serial, NotConfiguredError> {
        self.serial.as_mut()
            .map_err(|err| *err)
    }
}


#[derive(Debug)]
pub enum TestStandInitError {
    Inner(host_lib::test_stand::TestStandInitError),
}
