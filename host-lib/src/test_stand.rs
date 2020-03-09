use std::sync::{
    LockResult,
    Mutex,
    MutexGuard,
};

use lazy_static::lazy_static;


/// An instance of the test stand
///
/// Holds all the resources that a test case might require.
pub struct TestStand {
    _guard: LockResult<MutexGuard<'static, ()>>,
}

impl TestStand {
    /// Create a new instance of `TestStand`
    pub fn new() -> Self {
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

        Self {
            _guard: guard,
        }
    }
}
