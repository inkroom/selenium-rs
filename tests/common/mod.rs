use std::time::Duration;

use selenium::{driver::Driver, option::FirefoxBuilder, SError};

pub fn new_driver() -> Driver {
    let option = if std::env::var("HEADLESS").is_ok() {
        FirefoxBuilder::new()
            .host("127.0.0.1")
            .port(4892)
            .head_leass()
    } else {
        FirefoxBuilder::new().driver(
            format!(
                "{}/geckodriver",
                std::env::current_dir()
                    .map_err(|f| SError::message(f.to_string()))
                    .unwrap()
                    .display()
            )
            .as_str(),
        )
    };

    let d = Driver::new(option.build()).unwrap();
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
