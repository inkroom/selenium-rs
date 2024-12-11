use std::{thread::sleep, time::Duration};

use selenium::By;

mod common;
#[test]
fn click() {
    let driver = common::new_driver();
    let ele = driver.find_element(By::Css("#click")).unwrap();

    driver
        .actions()
        .move_pointer(&ele)
        .click(None)
        .perform()
        .unwrap();
    assert_eq!("已点击",ele.get_property("innerHTML").unwrap());
}
