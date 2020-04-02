use std::{
    io,
    time::{
        Duration,
        Instant,
    },
};

use serialport::{
    self,
    SerialPort,
    SerialPortSettings,
};

use crate::Error;


/// A Serial-to-USB converter that is connected to the test target
pub struct Serial {
    port: Box<dyn SerialPort>,
}

impl Serial {
    /// Open a serial connection
    ///
    /// `path` is the path to the serial device file.
    pub fn new(path: &str) -> Result<Self, SerialInitError> {
        let port =
            serialport::open_with_settings(
                path,
                // The configuration is hardcoded for now. We might want to load
                // this from the configuration file later.
                &SerialPortSettings {
                    baud_rate: 115200,
                    .. SerialPortSettings::default()
                }
            )
            .map_err(|err| SerialInitError(err))?;

        Ok(
            Self {
                port,
            }
        )
    }

    /// Send raw byte data through the serial connection
    pub fn send(&mut self, data: &[u8]) -> Result<(), SerialSendError> {
        self.port.write_all(data)
            .map_err(|err| SerialSendError(err))
    }

    /// Wait to receive the provided message
    ///
    /// Returns the receive buffer, once the message was received. Returns an
    /// error, if it times out before that, or an I/O error occurs.
    pub fn wait_for(&mut self, message: &[u8], timeout: Duration)
        -> Result<Vec<u8>, SerialWaitError>
    {
        self.wait_for_inner(message, timeout)
            .map_err(|err| SerialWaitError(err))
    }

    fn wait_for_inner(&mut self, message: &[u8], timeout: Duration)
        -> Result<Vec<u8>, Error>
    {
        let mut buf   = Vec::new();
        let     start = Instant::now();

        self.port.set_timeout(timeout)?;

        loop {
            if buf.ends_with(message) {
                return Ok(buf);
            }
            if start.elapsed() > timeout {
                return Err(io::Error::from(io::ErrorKind::TimedOut).into());
            }

            // Read one more byte. This might seem tedious, but it's the only
            // way I could think of that doesn't require allocating a finite
            // buffer (forcing a decision of how big that buffer is going to
            // be), or complicating the interface by forcing the user to pass a
            // buffer.
            //
            // It's probably not very efficient, but seems good enough.
            buf.push(0);
            let len = buf.len();
            self.port.read_exact(&mut buf[len - 1..])?;
        }
    }
}


/// Error initializing the serial connection
#[derive(Debug)]
pub struct SerialInitError(pub serialport::Error);

/// Error sending data through the serial connection
#[derive(Debug)]
pub struct SerialSendError(pub io::Error);


/// Error receiving data from the serial connection
#[derive(Debug)]
pub struct SerialWaitError(pub Error);
