//! The library code that supports this test suite
//!
//! For now, this is just all custom code used by this test suite (except for
//! code shared with the firmware). Eventually, a lot of it will be moved into
//! a generally usable library that can be shared with other test suites.


pub mod result;
pub mod serial;
pub mod target;


pub use self::result::{
    Result,
    Error,
};


use self::{
    serial::Serial,
    target::Target,
};


use std::{
    fs::File,
    io::prelude::*,
};

use serde::Deserialize;


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
        // Read configuration file
        let mut config = Vec::new();
        File::open("test-stand.toml")?
            .read_to_end(&mut config)?;

        // Parse configuration file
        let config: Config = toml::from_slice(&config)?;

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


#[derive(Deserialize)]
struct Config {
    target: String,
    serial: String,
}
