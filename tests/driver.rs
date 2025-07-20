use std::collections::HashMap;

use crate::common::sleep;
mod common;

#[test]
fn find_element() {
    let driver = common::new_driver();

    let v = driver.find_element(selenium::By::Css(".not_exist"));

    assert!(
        if let Err(selenium::SError::Http(status, _msg)) = v {
            status == 404
        } else {
            false
        },
        "not exist"
    );
}

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
fn execute_async_script() {
    let driver = common::new_driver();

    let r: String = driver.execute_async_script("setTimeout(()=> {arguments[arguments.length - 1]('result');} ,1000);  ", &[]).unwrap();
    
    assert_eq!("result", r);
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

    assert_eq!(
        "after alert",
        ele.get_property("innerHTML").unwrap().unwrap()
    );
}

#[test]
fn confirm() {
    let driver = common::new_driver();
    let ele = driver.find_element(selenium::By::Css("#confirm")).unwrap();
    ele.click().unwrap();

    driver.accept_alert().unwrap();

    assert_eq!(
        "yes confirm",
        ele.get_property("innerHTML").unwrap().unwrap()
    );

    ele.click().unwrap();

    driver.dismiss_alert().unwrap();

    assert_eq!(
        "no confirm",
        ele.get_property("innerHTML").unwrap().unwrap()
    );
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

    assert_eq!("text", ele.get_property("innerHTML").unwrap().unwrap());
}

#[test]
fn teke_screenshot() {
    let driver = common::new_driver();
    let v = driver.take_screenshot().unwrap();
    // 如果使用了隐私模式启动的话，会启动一个英文环境，导致截图出现中文乱码
    std::fs::write("target/screenshot.png", v).unwrap();
}

#[test]
fn test_switch_to_window() {
    let driver = common::new_driver();
    let w = driver.get_window_handles().unwrap();
    let nw = driver
        .new_window(selenium::driver::NewWindowType::Tab)
        .unwrap();

    driver.switch_to_window(&nw).unwrap();
    assert_eq!("about:blank", driver.get_current_url().unwrap());
    assert_eq!(2, driver.get_window_handles().unwrap().len());
    // 换回去
    driver.close_window().unwrap();
    driver.switch_to_window(w[0].as_str()).unwrap();

    assert_eq!(
        format!(
            "file://{}/tests/common/test.html",
            std::env::current_dir().unwrap().display()
        ),
        driver.get_current_url().unwrap()
    );

    assert_eq!(1, driver.get_window_handles().unwrap().len());
}
