use std::sync::{
    LockResult,
    Mutex,
    MutexGuard,
};

use lazy_static::lazy_static;

use crate::{
    assistant::Assistant,
    config::{
        Config,
        ConfigReadError,
    },
    conn::{
        Conn,
        ConnInitError,
    },
};


/// An instance of the test stand
///
/// Holds all the resources that a test case might require.
pub struct TestStand {
    /// Guarantees exclusive access to the test target
    ///
    /// Must not be dropped while this exclusive access is required. Once it is
    /// dropped, another test case might start running immediately.
    pub guard: LockResult<MutexGuard<'static, ()>>,

    /// Connection to the test target
    ///
    /// This field will be `Err`, if the test target has not been specified in
    /// the configuration file.
    pub target: Result<Conn, NotConfiguredError>,

    /// Connection to the test assistant
    ///
    /// This field will be `Err`, if the test assistant has not been specified
    /// in the configuration file.
    pub assistant: Result<Assistant, NotConfiguredError>,
}

impl TestStand {
    /// Create a new instance of `TestStand`
    pub fn new() -> Result<Self, TestStandInitError> {
        // By default, Rust runs tests in parallel on multiple threads. This can
        // be controlled through a command-line argument and an environment
        // variable, but there doesn't seem to be a way to configure this in
        // `Cargo.toml` or a configuration file.
        //
        // Let's just use a mutex here to prevent our tests from running in
        // parallel. The returned guard will be stored as a field, meaning the
        // mutex will be held until this struct is dropped. Concurrent
        // instantiations of this method will block here, until the `TestStand`
        // instance holding the mutex has been dropped.
        //
        // Please note that this returns a `Result` that we don't unwrap. Doing
        // so is not necessary, as the error case just tells us that another
        // thread holding this lock panicked. We don't care about that, as the
        // mutex is still acquired in that case.
        lazy_static! { static ref MUTEX: Mutex<()> = Mutex::new(()); }
        let guard = MUTEX.lock();

        let config = Config::read()
            .map_err(|err| TestStandInitError::ConfigRead(err))?;

        let mut target    = Err(NotConfiguredError("target"));
        let mut assistant = Err(NotConfiguredError("assistant"));

        if let Some(path) = config.target {
            target = Ok(
                Conn::new(&path)
                    .map_err(|err| TestStandInitError::ConnInit(err))?
            );
        }
        if let Some(path) = config.assistant {
            let conn = Conn::new(&path)
                .map_err(|err| TestStandInitError::ConnInit(err))?;
            assistant = Ok(Assistant::new(conn));
        }

        Ok(
            Self {
                guard,
                target,
                assistant,
            },
        )
    }
}


/// Error initializing the test stand
#[derive(Debug)]
pub enum TestStandInitError {
    /// Error reading configuration
    ConfigRead(ConfigReadError),

    /// Error initializing a serial connection
    ConnInit(ConnInitError),
}

/// The resource you tried to access was not specified in the configuration file
///
/// If something isn't specified the configuration file, it is not going to be
/// available.
#[derive(Clone, Copy, Debug)]
pub struct NotConfiguredError(pub &'static str);
