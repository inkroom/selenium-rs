use common::sleep;
use std::{collections::HashMap, time::Duration};
mod common;
#[test]
fn get_page_source() {
    let driver = common::new_driver();
    let source = driver.get_page_source().unwrap();

    // 获取到的源码被做了一些处理，导致跟原始文件不一样了，所以不能断言
    println!("{}", source);
}

#[test]
fn execute_script() {
    let driver = common::new_driver();

    let r: i32 = driver.execute_script("return 1;", &[]).unwrap();
    assert_eq!(1, r);
    let r: String = driver
        .execute_script("return '1'", vec![].as_slice())
        .unwrap();
    assert_eq!("1", r);
    let r: f32 = driver
        .execute_script("return 1.0;", vec![].as_slice())
        .unwrap();
    assert_eq!(1.0, r);
    let r: HashMap<String, String> = driver
        .execute_script("return {};", vec![].as_slice())
        .unwrap();
    assert_eq!(0, r.len());
    let r: Vec<String> = driver
        .execute_script("return ['1'];", vec![].as_slice())
        .unwrap();
    assert_eq!(1, r.len());
    assert_eq!("1", r[0]);
    // driver.execute_script("return document.getElementById('click');", vec![].as_slice()).unwrap();
    // driver.execute_script("return null;", vec![].as_slice()).unwrap();
    // driver.execute_script("return {'element-33224343':'32233'};", vec![].as_slice()).unwrap();
}

#[test]
fn timeout() {
    let driver = common::new_driver();

    driver
        .set_timeouts(selenium::TimeoutType::Implicit(2209))
        .unwrap();
    driver
        .set_timeouts(selenium::TimeoutType::PageLoad(2900))
        .unwrap();
    driver
        .set_timeouts(selenium::TimeoutType::Script(2900))
        .unwrap();

    let t = driver.get_timeouts().unwrap();
    assert_eq!(3, t.len());
    for ele in t {
        match ele {
            selenium::TimeoutType::Script(v) => assert_eq!(2900, v),
            selenium::TimeoutType::PageLoad(v) => assert_eq!(2900, v),
            selenium::TimeoutType::Implicit(v) => assert_eq!(2209, v),
        }
    }
}
#[test]
fn dismiss_alert() {
    let driver = common::new_driver();
    let ele = driver.find_element(selenium::By::Css("#alert")).unwrap();
    ele.click().unwrap();

    driver.dismiss_alert().unwrap();

    assert_eq!("after alert", ele.get_property("innerHTML").unwrap());
}

#[test]
fn confirm() {
    let driver = common::new_driver();
    let ele = driver.find_element(selenium::By::Css("#confirm")).unwrap();
    ele.click().unwrap();

    driver.accept_alert().unwrap();

    assert_eq!("yes confirm", ele.get_property("innerHTML").unwrap());

    ele.click().unwrap();

    driver.dismiss_alert().unwrap();

    assert_eq!("no confirm", ele.get_property("innerHTML").unwrap());
}

#[test]
fn alert_text() {
    let driver = common::new_driver();
    let ele = driver.find_element(selenium::By::Css("#alert")).unwrap();
    ele.click().unwrap();

    assert_eq!("1", driver.get_alert_text().unwrap());
}

#[test]
fn prompt() {
    let driver = common::new_driver();
    let ele = driver.find_element(selenium::By::Css("#prompt")).unwrap();
    ele.click().unwrap();

    driver.send_alert_text("text").unwrap();
    driver.accept_alert().unwrap();

    assert_eq!("text", ele.get_property("innerHTML").unwrap());
}

#[test]
fn teke_screenshot() {
    let driver = common::new_driver();
    let v = driver.take_screenshot().unwrap();
    // gui本身正常，截图出现了中文乱码，但是不确定和环境有没有关系，需要进一步测试
    std::fs::write("screenshot.png", v).unwrap();
}
