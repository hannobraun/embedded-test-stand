//! Test Suite for the USART API in LPC8xx HAL
//!
//! This test suite communicates with hardware. See top-level README.md for
//! wiring instructions.


use std::time::Duration;

use lpc845_test_suite::{
    Result,
    TestStand,
};


#[test]
fn it_should_send_messages() -> Result {
    let mut test_stand = TestStand::new()?;

    let message = b"Hello, world!";
    test_stand.target.send_usart(message)?;

    let timeout  = Duration::from_millis(50);
    let received = test_stand.assistant
        .receive_from_target_usart(message, timeout)?;

    assert_eq!(received, message);
    Ok(())
}

#[test]
fn it_should_receive_messages() -> Result {
    let mut test_stand = TestStand::new()?;

    let message = b"Hello, world!";
    test_stand.assistant.send_to_target_usart(message)?;

    let timeout  = Duration::from_millis(50);
    let received = test_stand.target.wait_for_usart_rx(message, timeout)?;

    assert_eq!(received, message);
    Ok(())
}

#[test]
fn it_should_send_messages_using_dma() -> Result {
    let mut test_stand = TestStand::new()?;

    let message = b"Hello, world!";
    test_stand.target.send_usart_dma(message)?;

    let timeout  = Duration::from_millis(50);
    let received = test_stand.assistant
        .receive_from_target_usart(message, timeout)?;

    assert_eq!(received, message);
    Ok(())
}

#[test]
fn it_should_receive_messages_via_dma() -> Result {
    let mut test_stand = TestStand::new()?;

    let message = b"Hello, world!";
    test_stand.assistant.send_to_target_usart_dma(message)?;

    let timeout  = Duration::from_millis(50);
    let received = test_stand.target.wait_for_usart_rx_dma(message, timeout)?;

    assert_eq!(received, message);
    Ok(())
}

#[test]
fn it_should_send_using_flow_control() -> Result {
    let mut test_stand = TestStand::new()?;

    test_stand.assistant.disable_cts()?;

    let message = b"Hello, world!";
    test_stand.target.send_usart_with_flow_control(message)?;

    let timeout = Duration::from_millis(50);
    test_stand.assistant.expect_nothing_from_target(timeout)?;

    test_stand.assistant.enable_cts()?;

    let timeout = Duration::from_millis(50);
    let received = test_stand.assistant
        .receive_from_target_usart(message, timeout)?;

    assert_eq!(received, message);
    Ok(())
}

#[test]
fn it_should_send_in_sync_mode() -> Result {
    let mut test_stand = TestStand::new()?;

    let message = b"Hello, world!";
    test_stand.target.send_usart_sync(message)?;

    let timeout  = Duration::from_millis(50);
    let received = test_stand.assistant
        .receive_from_target_usart_sync(message, timeout)?;

    assert_eq!(received, message);
    Ok(())
}

#[test]
fn it_should_receive_in_sync_mode() -> Result {
    let mut test_stand = TestStand::new()?;

    let message = b"Hello, world!";
    test_stand.assistant.send_to_target_usart_sync(message)?;

    let timeout  = Duration::from_millis(50);
    let received = test_stand.target.wait_for_usart_rx_sync(message, timeout)?;

    assert_eq!(received, message);
    Ok(())
}

#[test]
fn it_should_ignore_received_data_until_an_address_is_matched() -> Result {
    let mut test_stand = TestStand::new()?;

    let address = b'X';
    let message = b"Hello, world!";

    test_stand.target.wait_for_address(address)?;

    // Send data that the receiver shouldn't pass on, trying to trick it in
    // various ways.
    test_stand.assistant.send_to_target_usart(b"111")?;
    test_stand.assistant.send_to_target_usart(&[address])?; // MSB not set
    test_stand.assistant.send_to_target_usart(b"222")?;
    test_stand.assistant.send_to_target_usart(&[b'Y' | 0x80])?; // wrong address
    test_stand.assistant.send_to_target_usart(b"333")?;

    // Now send the address, plus the data that should actually arrive.
    test_stand.assistant.send_to_target_usart(&[address | 0x80])?;
    test_stand.assistant.send_to_target_usart(message)?;

    let timeout = Duration::from_millis(50);
    let received = test_stand.target.wait_for_usart_rx(message, timeout)?;

    assert_eq!(received, message);
    Ok(())
}
