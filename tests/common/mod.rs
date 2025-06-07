use std::time::Duration;

use selenium::{
    driver::Driver,
    option::{Browser, ChromeBuilder, EdgeBuilder, FirefoxBuilder, Proxy, SafariBuilder},
    SError,
};

fn use_firefox() -> Driver {
    let mut option = if std::env::var("HEADLESS").is_ok() {
        FirefoxBuilder::new().head_less()
    } else {
        FirefoxBuilder::new()
    }
    .timeout(1000)
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
    // .binary("/usr/bin/firefox")
    .private();
    if let Ok(url) = std::env::var("FIREFOX_URL") {
        option = option.url(url.as_str());
    }
    Driver::new(option.build()).unwrap()
}
fn get_available_port() -> u16 {
    std::net::TcpListener::bind("0.0.0.0:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}
fn use_chrome() -> Driver {
    Driver::new(
        if std::env::var("HEADLESS").is_ok() {
            ChromeBuilder::new().head_less()
        } else {
            ChromeBuilder::new()
        }
        .driver(std::env::var("BROWSER_DRIVER").unwrap().as_str())
        .binary(std::env::var("BROWSER_BINARY").unwrap().as_str())
        .private()
        .timeout(1000)
        .add_argument("--no-zygote")
        .add_argument("--disable-gpu")
        .add_argument(format!("--remote-debugging-port={}", get_available_port()).as_str())
        .add_argument("--disable-dev-shm-usage")
        .add_argument("--no-sandbox")
        .build(),
    )
    .unwrap()
}
/// 因为mac的设计问题，当quit的时候程序并没有完全退出，而且如果启用了多个session，总会有各种问题，总之就是safari的driver有些问题，所以只跑单个test没问题，一起并发跑就不行
/// 使用 RUST_TEST_THREADS=1 cargo test 限制单线程
fn use_safari() -> Driver {
    // 需要睡眠等待driver处理
    sleep(3);
    let v = Driver::new(
        SafariBuilder::new()
            .url("http://127.0.0.1:48273")
            .timeout(1000)
            .build(),
    )
    .unwrap();
    v.set_timeouts(selenium::TimeoutType::Implicit(100))
        .unwrap();
    v.set_timeouts(selenium::TimeoutType::PageLoad(1500))
        .unwrap();
    v
}

fn use_edge() -> Driver {
    Driver::new(
        if std::env::var("HEADLESS").is_ok() {
            EdgeBuilder::new().head_less()
        } else {
            EdgeBuilder::new()
        }
        .timeout(1000)
        .driver(std::env::var("BROWSER_DRIVER").unwrap().as_str())
        .binary(std::env::var("BROWSER_BINARY").unwrap().as_str())
        .private()
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
        "safari" => use_safari(),
        "edge" => use_edge(),
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
    if let Browser::Safari = d.browser() {
        sleep(2);
    }
    d
}

pub fn sleep(time: u64) {
    std::thread::sleep(Duration::from_secs(time));
}
