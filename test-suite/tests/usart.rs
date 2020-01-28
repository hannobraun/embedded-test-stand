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
