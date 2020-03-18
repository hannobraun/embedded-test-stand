use host_lib::{
    serial::Serial,
    test_stand::TestStandInitError,
};

use super::{
    assistant::Assistant,
    target::Target,
};


/// An instance of the test stand
///
/// Used to access all resources that a test case requires.
pub struct TestStand(host_lib::TestStand);

impl TestStand {
    /// Initializes the test stand
    ///
    /// Reads the `test-stand.toml` configuration file and initializes test
    /// stand resources, as configured in there.
    pub fn new() -> Result<Self, TestStandInitError> {
        let test_stand = host_lib::TestStand::new()?;
        Ok(TestStand(test_stand))
    }

    /// Returns the connection to the test target (device under test)
    pub fn target(&mut self) -> Result<Target, NotConfiguredError> {
        match &mut self.0.target {
            Some(target) => Ok(Target(target)),
            None         => Err(NotConfiguredError("target")),
        }
    }

    /// Returns the connection to the test assistant
    pub fn assistant(&mut self) -> Result<Assistant, NotConfiguredError> {
        match &mut self.0.assistant {
            Some(assistant) => Ok(Assistant(assistant)),
            None            => Err(NotConfiguredError("assistant")),
        }
    }

    /// Returns the connection to the Serial-to-USB converter
    pub fn serial(&mut self) -> Result<&mut Serial, NotConfiguredError> {
        match &mut self.0.serial {
            Some(serial) => Ok(serial),
            None         => Err(NotConfiguredError("serial")),
        }
    }
}


#[derive(Debug)]
pub struct NotConfiguredError(&'static str);
