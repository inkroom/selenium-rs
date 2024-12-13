use std::time::Duration;

use selenium::{driver::Driver, option::FirefoxBuilder, SError};

pub fn new_driver() -> Driver {
    let mut option = FirefoxBuilder::new().driver(
        format!(
            "{}/geckodriver",
            std::env::current_dir()
                .map_err(|f| SError::message(f.to_string()))
                .unwrap()
                .display()
        )
        .as_str(),
    );
    if std::env::var("HEADLESS").is_ok() {
        option = option.head_leass();
    }
    let option = option.build();

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
