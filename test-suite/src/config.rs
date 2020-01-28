use std::{
    fs::File,
    io::prelude::*,
};

use serde::Deserialize;

use super::Result;


#[derive(Deserialize)]
pub struct Config {
    pub target: String,
    pub serial: String,
}

impl Config {
    pub fn read() -> Result<Self> {
        // Read configuration file
        let mut config = Vec::new();
        File::open("test-stand.toml")?
            .read_to_end(&mut config)?;

        // Parse configuration file
        let config = toml::from_slice(&config)?;

        Ok(config)
    }
}
