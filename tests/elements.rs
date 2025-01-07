use selenium::By;

mod common;
#[test]
fn get_attribute() {
    let driver = common::new_driver();
    let ele = driver.find_element(By::Css("#checkbox")).unwrap();

    assert_eq!("1", ele.get_attribute("value").unwrap().unwrap());

    assert_eq!(None, ele.get_attribute("href").unwrap());
    println!("[{:?}]", ele.get_attribute("href").unwrap());
}
#[test]
fn get_property() {
    let driver = common::new_driver();
    let ele = driver.find_element(By::Css("#checkbox")).unwrap();

    assert_eq!("1", ele.get_property("value").unwrap().unwrap());
    assert_eq!(None, ele.get_property("ok").unwrap());

    let ele = driver.find_element(By::Id("href")).unwrap();
    assert_eq!(
        Some(format!(
            "file://{}/tests/2329",
            std::env::current_dir().unwrap().display()
        )),
        ele.get_property("href").unwrap()
    );

    let ele = driver.find_element(By::Id("src")).unwrap();
    assert_eq!(
        Some(format!(
            "file://{}/tests/common/1.png",
            std::env::current_dir().unwrap().display()
        )),
        ele.get_property("src").unwrap()
    );
}

#[test]
fn get_css() {
    let driver = common::new_driver();
    let ele = driver.find_element(By::Css("#label")).unwrap();

    match driver.browser() {
        selenium::option::Browser::Chrome => {
            assert_eq!("rgba(255, 0, 0, 1)", ele.get_css_value("color").unwrap())
        }
        selenium::option::Browser::Edge => {
            assert_eq!("rgba(255, 0, 0, 1)", ele.get_css_value("color").unwrap())
        }
        _ => {
            assert_eq!("rgb(255, 0, 0)", ele.get_css_value("color").unwrap())
        }
    }
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
    match driver.browser() {
        selenium::option::Browser::Safari => {
            assert_eq!("P", ele.get_tag_name().unwrap());
        }
        _ => {
            assert_eq!("p", ele.get_tag_name().unwrap());
        }
    }
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

    assert!(!ele.is_enabled().unwrap());
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

    assert_eq!("测试clear", ele.get_property("value").unwrap().unwrap());

    ele.clear().unwrap();

    assert_eq!("", ele.get_property("value").unwrap().unwrap());
}

#[test]
fn send_keys() {
    let driver = common::new_driver();
    let ele = driver.find_element(By::Css("#clear")).unwrap();
    ele.clear().unwrap();
    ele.send_keys("demo测试").unwrap();

    assert_eq!("demo测试", ele.get_property("value").unwrap().unwrap());
}

#[test]
fn take_screenshot() {
    let driver = common::new_driver();
    let ele = driver.find_element(By::Css("#label")).unwrap();

    std::fs::write("element_screenshot.png", ele.take_screenshot().unwrap()).unwrap();
}

#[test]
fn is_displayed() {
    let driver = common::new_driver();
    let ele = driver.find_element(By::Css("#clear")).unwrap();

    assert_eq!(true, ele.is_displayed().unwrap());
    assert_eq!(
        false,
        driver
            .find_element(By::Css("#is_displayed_false"))
            .unwrap()
            .is_displayed()
            .unwrap()
    );
}
