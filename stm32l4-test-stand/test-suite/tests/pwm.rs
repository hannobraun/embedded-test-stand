//! Test Suite for the PWM functionality of the target hardware
//!
//! This test suite communicates with hardware. See top-level README.md for
//! wiring instructions.


use std::time::Duration;

use stm32l4_test_suite::{
    Result,
    TestStand,
};


#[test]
fn it_should_create_a_pwm_signal() -> Result {
    let mut test_stand = TestStand::new()?;

    let period_ms = 10_u32;

    // When `_interrupt` is dropped, the PWM signal will be stopped.
    let _interrupt = test_stand.target.start_pwm_signal()?;

    let timeout = Duration::from_millis((period_ms * 2).into());
    let measurement = test_stand.assistant.measure_pwm_signal(5, timeout)?;

    let min_acceptable = Duration::from_millis((period_ms *  9/10).into());
    let max_acceptable = Duration::from_millis((period_ms * 11/10).into());

    assert!(measurement.min >= min_acceptable);
    assert!(measurement.max <= max_acceptable);

    Ok(())
}
