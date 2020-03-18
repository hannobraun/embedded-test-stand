//! The library code that supports this test suite
//!
//! For now, this is just all custom code used by this test suite (except for
//! code shared with the firmware). Eventually, a lot of it will be moved into
//! a generally usable library that can be shared with other test suites.


pub mod assistant;
pub mod error;
pub mod target;
pub mod test_stand;


pub use self::{
    error::{
        Error,
        Result,
    },
    test_stand::TestStand,
};
