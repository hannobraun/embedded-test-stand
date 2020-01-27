//! Test Suite for the USART API in LPC8xx HAL
//!
//! This test suite requires two serial connections:
//! - To the test target, to control the device's behavior.
//! - To the target's USART instance used for the test, via a USB/Serial
//!   converter.
//!
//! The configuration file (`test-stand.toml`) is used to determine which serial
//! ports to connect to for each purpose. You probably need to update it, to
//! reflect the realities on your system.


use std::time::Duration;

use lpc845_test_suite::{
    Result,
    TestStand,
};


#[test]
fn it_should_send_messages() -> Result {
    let mut test_stand = TestStand::new()?;

    let message = b"Hello, world!";
    test_stand.target().send_usart(message)?;

    let timeout  = Duration::from_millis(50);
    let received = test_stand.serial().wait_for(message, timeout)?;

    assert_eq!(received, message);
    Ok(())
}
