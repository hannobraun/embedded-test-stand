//! Test Suite for the GPIO API in STM32L4xx HAL
//!
//! This test suite communicates with hardware. See top-level README.md for
//! wiring instructions.


use stm32l4_test_suite::{
    Result,
    TestStand,
};


#[test]
fn it_should_set_pin_level() -> Result {
    let mut test_stand = TestStand::new()?;

    test_stand.target.set_pin_low()?;
    assert!(test_stand.assistant.pin_is_low()?);

    test_stand.target.set_pin_high()?;
    assert!(test_stand.assistant.pin_is_high()?);

    Ok(())
}

#[test]
fn it_should_read_input_level() -> Result {
    let mut test_stand = TestStand::new()?;

    test_stand.assistant.set_pin_low()?;
    assert!(test_stand.target.pin_is_low()?);

    test_stand.assistant.set_pin_high()?;
    assert!(test_stand.target.pin_is_high()?);

    Ok(())
}
