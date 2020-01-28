//! The library code that supports this test suite
//!
//! For now, this is just all custom code used by this test suite (except for
//! code shared with the firmware). Eventually, a lot of it will be moved into
//! a generally usable library that can be shared with other test suites.


pub mod result;
pub mod serial;


pub use self::result::{
    Result,
    Error,
};


use self::serial::Serial;


use std::{
    fs::File,
    io::{
        self,
        prelude::*,
    },
};

use serde::Deserialize;
use serialport::{
    self,
    SerialPort,
    SerialPortSettings,
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


/// The test suite's connection to the test target (device under test)
pub struct Target {
    port: Box<dyn SerialPort>,
}

impl Target {
    /// Open a connection to the target
    fn new(path: &str) -> serialport::Result<Self> {
        let port = serialport::open_with_settings(
            path,
            // The configuration is hardcoded for now. We might want to load
            // this from the configuration file later.
            &SerialPortSettings {
                baud_rate: 115200,
                .. SerialPortSettings::default()
            }
        )?;

        // Use a clone of the serialport, so `Serial` can use the same port.
        let port = port.try_clone()?;

        Ok(
            Self {
                port,
            }
        )
    }

    /// Instruct the target to send this message via USART
    pub fn send_usart(&mut self, message: &[u8]) -> io::Result<()> {
        // This works fine for now, as the test firmware just echos what it
        // receives, and all we check is whether it did so. To write any more
        // test cases, we're going to need a bit more structure here.
        self.port.write_all(message)
    }
}
