use host_lib::{
    config::{
        Config,
        ConfigReadError,
    },
    serial::{
        Serial,
        SerialInitError,
    },
    target::TargetInitError,
};

use super::target::Target;


/// An instance of the test stand
///
/// Used to access all resources that a test case requires.
pub struct TestStand {
    _test_stand: host_lib::TestStand,

    target: host_lib::Target,
    serial: Serial,
}

impl TestStand {
    /// Initializes the test stand
    ///
    /// Reads the `test-stand.toml` configuration file and initializes test
    /// stand resources, as configured in there.
    pub fn new() -> Result<Self, TestStandInitError> {
        let test_stand = host_lib::TestStand::new();

        let config = Config::read()
            .map_err(|err| TestStandInitError::ConfigRead(err))?;

        let target = host_lib::Target::new(&config.target)
            .map_err(|err| TestStandInitError::TargetInit(err))?;
        let serial = Serial::new(&config.serial)
            .map_err(|err| TestStandInitError::SerialInit(err))?;

        Ok(
            TestStand {
                _test_stand: test_stand,

                target,
                serial,
            }
        )
    }

    /// Returns the connection to the test target (device under test)
    pub fn target(&mut self) -> Target {
        Target::new(&mut self.target)
    }

    /// Returns the connection to the Serial-to-USB converter
    pub fn serial(&mut self) -> &mut Serial {
        &mut self.serial
    }
}


#[derive(Debug)]
pub enum TestStandInitError {
    ConfigRead(ConfigReadError),
    SerialInit(SerialInitError),
    TargetInit(TargetInitError),
}
