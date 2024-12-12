use std::time::Duration;

use selenium::{driver::Driver, option::FirefoxBuilder, SError};

pub fn new_driver() -> Driver {
    let option = FirefoxBuilder::new()
        .driver(
            format!(
                "{}/geckodriver",
                std::env::current_dir()
                    .map_err(|f| SError::message(f.to_string()))
                    .unwrap()
                    .display()
            )
            .as_str(),
        )
        .add_env("DISPLAY", ":1")
        .add_env("LC_ALL", "zh_CN.UTF-8")
        .add_env("LANG", "C.UTF-8")
        .private()
        .build();

    let d = Driver::new(option).unwrap();
    d.get(
        format!(
            "file://{}/tests/common/test.html",
            std::env::current_dir()
                .map_err(|f| SError::message(f.to_string()))
                .unwrap()
                .display()
        )
        .as_str(),
    )
    .unwrap();
    d
}

pub fn sleep(time: u64) {
    std::thread::sleep(Duration::from_secs(time));
}
