use std::{collections::HashMap, thread::sleep, time::Duration};

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
