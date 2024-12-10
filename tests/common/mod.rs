use std::{thread::sleep, time::Duration};

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
    sleep(Duration::from_secs(5));
    d
}
