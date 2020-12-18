//! Test Suite for timer interrupt functionality in the target hardware
//!
//! This test suite communicates with hardware. See top-level README.md for
//! wiring instructions.


use std::time::Duration;

use stm32l4_test_suite::{
    Result,
    TestStand,
};


#[test]
fn it_should_fire_regular_timer_interrupts() -> Result {
    let mut test_stand = TestStand::new()?;

    let period_ms = 10;

    // When `_interrupt` is dropped, the timer interrupt will be stopped.
    let _interrupt = test_stand.target.start_timer_interrupt(period_ms)?;

    let timeout = Duration::from_millis((period_ms * 2).into());
    let measurement = test_stand.assistant.measure_timer_interrupt(5, timeout)?;

    let min_acceptable = Duration::from_millis((period_ms *  9/10).into());
    let max_acceptable = Duration::from_millis((period_ms * 11/10).into());

    assert!(measurement.min >= min_acceptable);
    assert!(measurement.max <= max_acceptable);

    Ok(())
}
