//! Test Suite for the ADC API in STM32L4xx HAL


use stm32l4_test_suite::{
    Result,
    TestStand,
};


#[test]
fn it_should_read_adc_values() -> Result {
    let mut test_stand = TestStand::new()?;

    test_stand.assistant.set_pin_5_low()?;
    let value = test_stand.target.read_adc()?;
    println!("value (low): {}", value);
    assert!(value < 16);

    test_stand.assistant.set_pin_5_high()?;
    let value = test_stand.target.read_adc()?;
    println!("value (high): {}", value);
    assert!(value > 2u16.pow(12) - 128);

    Ok(())
}
