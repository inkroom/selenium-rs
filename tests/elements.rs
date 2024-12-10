use std::{thread::sleep, time::Duration};

use selenium::By;

mod common;
#[test]
fn get_attribute() {
    let driver = common::new_driver();
    let ele = driver.find_element(By::Css("#checkbox")).unwrap();

    assert_eq!("1", ele.get_attribute("value").unwrap());
}
#[test]
fn get_property() {
    let driver = common::new_driver();
    let ele = driver.find_element(By::Css("#checkbox")).unwrap();

    assert_eq!("1", ele.get_attribute("value").unwrap());
}

#[test]
fn get_css() {
    let driver = common::new_driver();
    let ele = driver.find_element(By::Css("#label")).unwrap();

    assert_eq!("rgb(255, 0, 0)", ele.get_css_value("color").unwrap());
}

#[test]
fn get_text() {
    let driver = common::new_driver();
    let ele = driver.find_element(By::Css("#p")).unwrap();

    assert_eq!("测试文字", ele.get_text().unwrap());
}

#[test]
fn get_tag_name() {
    let driver = common::new_driver();
    let ele = driver.find_element(By::Css("#p")).unwrap();

    assert_eq!("p", ele.get_tag_name().unwrap());
}

#[test]
fn get_rect() {
    let driver = common::new_driver();
    let ele = driver.find_element(By::Css("#rect")).unwrap();
    let rect = ele.get_rect().unwrap();
    println!("{:?}", rect);
    assert_eq!(29.0, rect.x.unwrap());
    assert_eq!(10.0, rect.y.unwrap());
    assert_eq!(48.0, rect.width.unwrap());
    assert_eq!(38.0, rect.height.unwrap());
    // assert_eq!("p",ele.get_tag_name().unwrap());
}

#[test]
fn is_enabled() {
    let driver = common::new_driver();
    let ele = driver.find_element(By::Css("#disabled")).unwrap();

    assert_eq!(false, ele.is_enabled().unwrap());
}

#[test]
fn click() {
    let driver = common::new_driver();
    let ele = driver.find_element(By::Css("#click")).unwrap();

    ele.click().unwrap();
}
#[test]
fn clear() {
    let driver = common::new_driver();
    let ele = driver.find_element(By::Css("#clear")).unwrap();

    assert_eq!("测试clear", ele.get_property("value").unwrap());

    ele.clear().unwrap();

    assert_eq!("", ele.get_property("value").unwrap());
}

