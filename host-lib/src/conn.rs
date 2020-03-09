use std::{
    slice,
    time::Duration,
};

use serde::{
    Deserialize,
    Serialize,
};
use serialport::{
    self,
    SerialPort,
    SerialPortSettings,
};

use crate::Error;


/// A connection to a firmware application
pub struct Conn {
    port: Box<dyn SerialPort>,
}

impl Conn {
    /// Open the connection
    pub fn new(path: &str) -> Result<Self, TargetInitError> {
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
            .map_err(|err| TargetInitError(err))?;

        // Use a clone of the serialport, so `Serial` can use the same port.
        let port = port.try_clone()
            .map_err(|err| TargetInitError(err))?;

        Ok(
            Self {
                port,
            }
        )
    }

    /// Send a message
    pub fn send<T>(&mut self, message: &T) -> Result<(), TargetSendError>
        where T: Serialize
    {
        self.send_inner(message)
            .map_err(|err| TargetSendError(err))
    }

    fn send_inner<T>(&mut self, message: &T) -> Result<(), Error>
        where T: Serialize
    {
        let mut buf = [0; 256];

        let serialized = postcard::to_slice_cobs(message, &mut buf)?;
        self.port.write_all(serialized)?;

        Ok(())
    }

    /// Receive a message
    pub fn receive<'de, T>(&mut self, timeout: Duration, buf: &'de mut Vec<u8>)
        -> Result<T, TargetReceiveError>
        where T: Deserialize<'de>
    {
        self.receive_inner(timeout, buf)
            .map_err(|err| TargetReceiveError(err))
    }

    fn receive_inner<'de, T>(&mut self,
        timeout: Duration,
        buf:     &'de mut Vec<u8>,
    )
        -> Result<T, Error>
        where T: Deserialize<'de>
    {
        self.port.set_timeout(timeout)?;

        loop {
            let mut b = 0; // initialized to `0`, but could be any value
            self.port.read_exact(slice::from_mut(&mut b))?;

            buf.push(b);

            if b == 0 {
                // We're using COBS encoding, so `0` signifies the end of the
                // message.
                break;
            }
        }

        let message = postcard::from_bytes_cobs(buf)?;
        Ok(message)
    }
}


#[derive(Debug)]
pub struct TargetInitError(serialport::Error);

#[derive(Debug)]
pub struct TargetSendError(Error);

#[derive(Debug)]
pub struct TargetReceiveError(Error);
