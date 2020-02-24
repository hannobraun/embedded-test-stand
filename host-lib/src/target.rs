use std::{
    sync::{
        LockResult,
        Mutex,
        MutexGuard,
    },
    time::Duration,
};

use lazy_static::lazy_static;
use serde::{
    Deserialize,
    Serialize,
};
use serialport::{
    self,
    SerialPort,
    SerialPortSettings,
};
use crate::{
    Error,
    receive,
};


/// The test suite's connection to the test target (device under test)
pub struct Target {
    port:   Box<dyn SerialPort>,
    _guard: LockResult<MutexGuard<'static, ()>>,
}

impl Target {
    /// Open a connection to the target
    pub fn new(path: &str) -> Result<Self, TargetInitError> {
        // By default, Rust runs tests in parallel on multiple threads. This can
        // be controlled through a command-line argument and an environment
        // variable, but there doesn't seem to be a way to configure this in
        // `Cargo.toml` or a configuration file.
        //
        // Let's just use a mutex here to prevent our tests from running in
        // parallel. The returned guard will be stored as a field, meaning the
        // mutex will be held until this struct is dropped. Concurrent
        // instantiations of this method will block here, until the `Target`
        // instance holding the mutex has been dropped.
        //
        // Please note that this returns a `Result` that we don't unwrap. Doing
        // so is not necessary, as the error case just tells us that another
        // thread holding this lock panicked. We don't care about that, as the
        // mutex is still acquired in that case.
        lazy_static! { static ref MUTEX: Mutex<()> = Mutex::new(()); }
        let guard = MUTEX.lock();

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
                _guard: guard,
            }
        )
    }

    /// Send a message to the test target
    pub fn send<T>(&mut self, message: &T) -> Result<(), TargetSendError>
        where T: Serialize
    {
        self.send_inner(message)
            .map_err(|err| TargetSendError(err))
    }

    pub fn send_inner<T>(&mut self, message: &T) -> Result<(), LowLevelError>
        where T: Serialize
    {
        let mut buf = [0; 256];

        let serialized = postcard::to_slice_cobs(message, &mut buf)?;
        self.port.write_all(serialized)?;

        Ok(())
    }

    /// Receive a message from the test target
    pub fn receive<'de, T>(&mut self, timeout: Duration, buf: &'de mut Vec<u8>)
        -> Result<T, TargetReceiveError>
        where T: Deserialize<'de>
    {
        self.port.set_timeout(timeout)
            .map_err(|err| TargetReceiveError(Error::from(err)))?;
        receive::<T, _>(&mut self.port, buf)
            .map_err(|err| TargetReceiveError(err))
    }
}


#[derive(Debug)]
pub struct TargetInitError(serialport::Error);

#[derive(Debug)]
pub struct TargetSendError(Error);

#[derive(Debug)]
pub struct TargetReceiveError(Error);
