use std::{
    io,
    sync::{
        LockResult,
        Mutex,
        MutexGuard,
    },
    time::{
        Duration,
        Instant,
    },
};

use lazy_static::lazy_static;
use serialport::{
    self,
    SerialPort,
    SerialPortSettings,
};
use lpc845_messages::{
    Event,
    Request,
};
use host_lib::{
    receive,
    send,
};

use crate::result::LowLevelError;


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

    /// Instruct the target to send this message via USART
    pub fn send_usart(&mut self, message: &[u8])
        -> Result<(), TargetSendError>
    {
        let mut buf = [0; 256];
        send(&Request::SendUsart(message), &mut self.port, &mut buf)
            .map_err(|err| TargetSendError(err))?;
        Ok(())
    }

    /// Wait to receive the provided data via USART
    ///
    /// Returns the receive buffer, once the data was received. Returns an
    /// error, if it times out before that, or an I/O error occurs.
    pub fn wait_for_usart_rx(&mut self, data: &[u8], timeout: Duration)
        -> Result<Vec<u8>, TargetUsartWaitError>
    {
        self.wait_for_usart_rx_inner(data, timeout)
            .map_err(|err| TargetUsartWaitError(err))
    }

    fn wait_for_usart_rx_inner(&mut self, data: &[u8], timeout: Duration)
        -> Result<Vec<u8>, LowLevelError>
    {
        let mut buf   = Vec::new();
        let     start = Instant::now();

        self.port.set_timeout(timeout)?;

        loop {
            if buf.windows(data.len()).any(|window| window == data) {
                return Ok(buf);
            }
            if start.elapsed() > timeout {
                return Err(io::Error::from(io::ErrorKind::TimedOut).into());
            }

            let mut tmp   = Vec::new();
            let     event = receive::<Event, _>(&mut self.port, &mut tmp)?;

            match event {
                Event::UsartReceive(data) => buf.extend(data),
            }
        }
    }
}


#[derive(Debug)]
pub struct TargetInitError(serialport::Error);

#[derive(Debug)]
pub struct TargetSendError(host_lib::Error);

#[derive(Debug)]
pub struct TargetUsartWaitError(LowLevelError);
