use std::time::Duration;

use selenium::{
    driver::Driver,
    option::{ChromeBuilder, FirefoxBuilder},
    SError,
};

fn use_firefox() -> Driver {
    Driver::new(
        if std::env::var("HEADLESS").is_ok() {
            FirefoxBuilder::new()
                .driver(
                    format!(
                        "{}/geckodriver",
                        std::env::current_dir()
                            .map_err(|f| SError::Message(f.to_string()))
                            .unwrap()
                            .display()
                    )
                    .as_str(),
                )
                .head_leass()
        } else {
            FirefoxBuilder::new()
            // .url("http://127.0.0.1:4845")
            .driver(
                format!(
                    "{}/geckodriver",
                    std::env::current_dir()
                        .map_err(|f| SError::Message(f.to_string()))
                        .unwrap()
                        .display()
                )
                .as_str(),
            )
        }
        .private()
        .build(),
    )
    .unwrap()
}

fn use_chrome() -> Driver {
    Driver::new(
        if std::env::var("HEADLESS").is_ok() {
            ChromeBuilder::new()
                .driver(
                    std::env::var("BROWSER_DRIVER").unwrap()
                    .as_str(),
                ).binary(std::env::var("BROWSER_BINARY").unwrap().as_str())
                .head_leass()
        } else {
            ChromeBuilder::new()
            .url("http://127.0.0.1:13324")
            .driver(
                std::env::var("BROWSER_DRIVER").unwrap().as_str()
            ).binary(std::env::var("BROWSER_BINARY").unwrap().as_str())
        }
        .add_argument("--no-zygote")
        .add_argument("--disable-gpu")
        // .add_argument("--remote-debugging-port=9222")
        .add_argument("--disable-dev-shm-usage")
        .add_argument("--no-sandbox")
        .build(),
    )
    .unwrap()
}

pub fn new_driver() -> Driver {
    let d = match std::env::var("BROWSER")
        .unwrap_or("firefox".to_string())
        .as_str()
    {
        "firefox" => use_firefox(),
        "chrome" => use_chrome(),
        _ => use_firefox(),
    };
    d.get(
        format!(
            "file://{}/tests/common/test.html",
            std::env::current_dir()
                .map_err(|f| SError::Message(f.to_string()))
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
