use std::time::Duration;

use self::lib::TestStand;


#[test]
fn it_should_send_messages() -> lib::Result {
    let mut test_stand = TestStand::new()?;

    let message = b"Hello, world!";
    test_stand.target().send_usart(message)?;

    let timeout  = Duration::from_millis(50);
    let received = test_stand.serial().wait_for(message, timeout)?;

    assert_eq!(received, message);
    Ok(())
}


/// The library code that supports this test suite
///
/// For now, this is just all custom code used by this test suite (except for
/// code shared with the firmware). Eventually, a lot of it will be moved into
/// a generally usable library that can be shared with other test suites.
mod lib {
    use std::{
        fs::File,
        io::{
            self,
            prelude::*,
        },
        time::{
            Duration,
            Instant,
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
            // receives, and all we check is whether it did so. To write any
            // more test cases, we're going to need a bit more structure here.
            self.port.write_all(message)
        }
    }


    /// A Serial-to-USB converter that is connected to the device under test
    pub struct Serial {
        port: Box<dyn SerialPort>,
    }

    impl Serial {
        /// Open a serial connection
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

            Ok(
                Self {
                    port,
                }
            )
        }

        /// Wait to receive the provided message
        ///
        /// Returns the receive buffer, once the message was received. Returns
        /// an error, if it times out before that, or an I/O error occurs.
        pub fn wait_for(&mut self, message: &[u8], timeout: Duration)
            -> Result<Vec<u8>>
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

                // Read one more byte. This might seem tedious, but it's the
                // only way I could think of that doesn't require allocating a
                // finite buffer (forcing a decision of how big that buffer is
                // going to be), or complicating the interface by forcing the
                // user to pass a buffer.
                //
                // It's probably not very efficient, but seems good enough.
                buf.push(0);
                let len = buf.len();
                self.port.read_exact(&mut buf[len - 1..])?;
            }
        }
    }


    /// Result type specific to this test suite
    pub type Result<T = ()> = std::result::Result<T, Error>;


    /// Error type specific to this test suite
    #[derive(Debug)]
    pub enum Error {
        Config(toml::de::Error),
        Io(io::Error),
        Serial(serialport::Error),
    }

    impl From<toml::de::Error> for Error {
        fn from(err: toml::de::Error) -> Self {
            Self::Config(err)
        }
    }

    impl From<io::Error> for Error {
        fn from(err: io::Error) -> Self {
            Self::Io(err)
        }
    }

    impl From<serialport::Error> for Error {
        fn from(err: serialport::Error) -> Self {
            Self::Serial(err)
        }
    }
}
