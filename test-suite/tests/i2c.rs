//! Test Suite for the I2C API in LPC8xx HAL
//!
//! This test suite communicates with hardware. See top-level README.md for
//! wiring instructions.


use std::time::Duration;

use lpc845_test_suite::{
    Result,
    TestStand,
};


#[test]
fn it_should_start_a_transaction() -> Result {
    let mut test_stand = TestStand::new()?;

    let data = 0x22;
    let timeout = Duration::from_millis(50);
    let reply = test_stand.target.start_i2c_transaction(data, timeout)?;

    assert_eq!(reply, data << 1);

    Ok(())
}
