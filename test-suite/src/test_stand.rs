use super::{
    Result,
    config::Config,
    serial::Serial,
    target::Target,
};


/// An instance of the test stand
///
/// Used to access all resources that a test case requires.
pub struct TestStand {
    target: Target,
    serial: Serial,
}

impl TestStand {
    /// Initializes the test stand
    ///
    /// Reads the `test-stand.toml` configuration file and initializes test
    /// stand resources, as configured in there.
    pub fn new() -> Result<Self> {
        let config = Config::read()?;

        let target = Target::new(&config.target)?;
        let serial = Serial::new(&config.serial)?;

        Ok(
            TestStand {
                target,
                serial,
            }
        )
    }

    /// Returns the connection to the test target (device under test)
    pub fn target(&mut self) -> &mut Target {
        &mut self.target
    }

    /// Returns the connection to the Serial-to-USB converter
    pub fn serial(&mut self) -> &mut Serial {
        &mut self.serial
    }
}
