//! The library code that supports this test suite
//!
//! For now, this is just all custom code used by this test suite (except for
//! code shared with the firmware). Eventually, a lot of it will be moved into
//! a generally usable library that can be shared with other test suites.


pub mod config;
pub mod result;
pub mod serial;
pub mod target;
pub mod test_stand;


pub use self::{
    result::{
        Error,
        Result,
    },
    test_stand::TestStand,
};
