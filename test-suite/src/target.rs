use std::io;

use serialport::{
    self,
    SerialPort,
    SerialPortSettings,
};


/// The test suite's connection to the test target (device under test)
pub struct Target {
    port: Box<dyn SerialPort>,
}

impl Target {
    /// Open a connection to the target
    pub fn new(path: &str) -> serialport::Result<Self> {
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
