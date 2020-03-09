use host_lib::{
    serial::Serial,
    test_stand::TestStandInitError,
};

use super::target::Target;


/// An instance of the test stand
///
/// Used to access all resources that a test case requires.
pub struct TestStand {
    test_stand: host_lib::TestStand,
}

impl TestStand {
    /// Initializes the test stand
    ///
    /// Reads the `test-stand.toml` configuration file and initializes test
    /// stand resources, as configured in there.
    pub fn new() -> Result<Self, TestStandInitError> {
        let test_stand = host_lib::TestStand::new()?;

        Ok(
            TestStand {
                test_stand,
            }
        )
    }

    /// Returns the connection to the test target (device under test)
    pub fn target(&mut self) -> Target {
        Target::new(&mut self.test_stand.target)
    }

    /// Returns the connection to the Serial-to-USB converter
    pub fn serial(&mut self) -> &mut Serial {
        &mut self.test_stand.serial
    }
}
