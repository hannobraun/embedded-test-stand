//! Test suite configuration


use std::{
    fs::File,
    io::prelude::*,
};

use serde::Deserialize;

use crate::Error;


/// The configuration options for the test suite
#[derive(Deserialize)]
pub struct Config {
    /// Path to the serial device connected to the test target
    pub target: Option<String>,

    /// Path to the serial device connected to the test assistant
    pub assistant: Option<String>,

    /// Path to the serial device connected to the USB/serial converter
    pub serial: Option<String>,
}

impl Config {
    /// Read configuration from the `test-stand.toml` file
    pub fn read() -> Result<Self, ConfigReadError> {
        Self::read_inner()
            .map_err(|err| ConfigReadError(err))
    }

    fn read_inner() -> Result<Self, Error> {
        // Read configuration file
        let mut config = Vec::new();
        File::open("test-stand.toml")?
            .read_to_end(&mut config)?;

        // Parse configuration file
        let config = toml::from_slice(&config)?;

        Ok(config)
    }
}


/// Error reading the configuration file
#[derive(Debug)]
pub struct ConfigReadError(pub Error);
