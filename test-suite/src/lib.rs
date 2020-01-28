//! The library code that supports this test suite
//!
//! For now, this is just all custom code used by this test suite (except for
//! code shared with the firmware). Eventually, a lot of it will be moved into
//! a generally usable library that can be shared with other test suites.


pub mod config;
pub mod result;
pub mod serial;
pub mod target;


pub use self::result::{
    Result,
    Error,
};


use self::{
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
