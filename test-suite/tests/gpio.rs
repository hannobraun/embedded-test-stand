use lpc845_test_suite::{
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
