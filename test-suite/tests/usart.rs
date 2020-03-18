//! Test Suite for the USART API in LPC8xx HAL
//!
//! This test suite requires two serial connections:
//! - To the test target, to control the device's behavior.
//! - To the test assistant, to send/receive message to/from the target's USART.
//!
//! The configuration file (`test-stand.toml`) is used to determine which serial
//! ports to connect to for each purpose. You probably need to update it, to
//! reflect the realities on your system.
//!
//! As of this writing, both the target and the assistant use PIO0_26 (pin 12 on
//! the LPC845-BRK) for receiving and PIO0_27 (pin 13) for sending. Please
//! connect the target and assistant accordingly.


use std::time::Duration;

use lpc845_test_suite::{
    Result,
    TestStand,
};


#[test]
fn it_should_send_messages() -> Result {
    let mut test_stand = TestStand::new()?;

    let message = b"Hello, world!";
    test_stand.target()?.send_usart(message)?;

    let timeout  = Duration::from_millis(50);
    let received = test_stand.assistant()?
        .receive_from_target_usart(message, timeout)?;

    assert_eq!(received, message);
    Ok(())
}

#[test]
fn it_should_receive_messages() -> Result {
    let mut test_stand = TestStand::new()?;

    let message = b"Hello, world!";
    test_stand.assistant()?.send_to_target_usart(message)?;

    let timeout  = Duration::from_millis(50);
    let received = test_stand.target()?.wait_for_usart_rx(message, timeout)?;

    assert_eq!(received, message);
    Ok(())
}
