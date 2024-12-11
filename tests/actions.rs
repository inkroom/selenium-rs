use common::sleep;
use selenium::{By, Key};

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
    assert_eq!("已点击", ele.get_property("innerHTML").unwrap());
}

#[test]
fn key() {
    let driver = common::new_driver();
    driver
        .actions()
        .key_down("a")
        .key_up("a")
        .perform()
        .unwrap();
    assert_eq!(
        "keydown=a 65",
        driver
            .find_element(By::Css("#demo"))
            .unwrap()
            .get_property("innerHTML")
            .unwrap()
    );
    assert_eq!(
        "keyup=a 65",
        driver
            .find_element(By::Css("#demo2"))
            .unwrap()
            .get_property("innerHTML")
            .unwrap()
    );
}

#[test]
fn pause() {
    let driver = common::new_driver();
    driver
        .actions()
        .key_down("a")
        .key_pause(100)
        .key_up("a")
        .perform()
        .unwrap();
    assert_eq!(
        "keydown=a 65",
        driver
            .find_element(By::Css("#demo"))
            .unwrap()
            .get_property("innerHTML")
            .unwrap()
    );
    assert_eq!(
        "keyup=a 65",
        driver
            .find_element(By::Css("#demo2"))
            .unwrap()
            .get_property("innerHTML")
            .unwrap()
    );
}

#[test]
fn key_special() {
    let driver = common::new_driver();
    driver
        .actions()
        .key_down_special(Key::ENTER)
        .key_pause(100)
        .key_up_special(Key::ENTER)
        .perform()
        .unwrap();

    assert_eq!(
        "keydown=Enter 13",
        driver
            .find_element(By::Css("#demo"))
            .unwrap()
            .get_property("innerHTML")
            .unwrap()
    );
    assert_eq!(
        "keyup=Enter 13",
        driver
            .find_element(By::Css("#demo2"))
            .unwrap()
            .get_property("innerHTML")
            .unwrap()
    );

    // 组合键
    driver
        .actions()
        .key_down_special(Key::CONTROL)
        .key_down("e")
        .key_up("e")
        .key_up_special(Key::CONTROL)
        .perform()
        .unwrap();
    sleep(20);
    assert_eq!(
        "keydown=e 69 ctrl",
        driver
            .find_element(By::Css("#demo"))
            .unwrap()
            .get_property("innerHTML")
            .unwrap()
    );
    assert_eq!(
        "keyup=Control 17",
        driver
            .find_element(By::Css("#demo2"))
            .unwrap()
            .get_property("innerHTML")
            .unwrap()
    );
}
