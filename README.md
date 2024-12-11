# selenium

[selenium](https://www.google.com/url?sa=t&source=web&rct=j&opi=89978449&url=https://www.selenium.dev/&ved=2ahUKEwj3zcTJ4ZyKAxWiQPUHHTlGFycQFnoECCMQAQ&usg=AOvVaw38IyEsg2ARkRX6lSh_KzqM)的rust实现

## 为何有此项目

- [crates.io](https://crates.io/search?q=selenium)上目前(2024-12)存在的两个项目([selenium-rs](https://crates.io/crates/selenium-rs),[selenium_webdriver](https://crates.io/crates/selenium_webdriver))，都是好几年前发了几个版本后就没动静了；
- 而且都存在一个问题，就是driver程序需要另外单独启动，没有集成到程序中，甚至用于通信的http地址都是写死在代码里的



## browser driver

目前只测试了firefox

| Browser           | Component                        |
| :---------------- | :------------------------------- |
| Chrome            | [chromedriver(.exe)](https://googlechromelabs.github.io/chrome-for-testing/#stable)     |
| Internet Explorer | [IEDriverServer.exe](https://www.selenium.dev/downloads/)    |
| Edge              | [MicrosoftWebDriver.msi](http://go.microsoft.com/fwlink/?LinkId=619687)   |
| Firefox           | [geckodriver(.exe)](https://github.com/mozilla/geckodriver/releases/) |
| Opera             | [operadriver(.exe)](https://github.com/operasoftware/operachromiumdriver/releases) |
| Safari            | [safaridriver](https://developer.apple.com/library/prerelease/content/releasenotes/General/WhatsNewInSafari/Articles/Safari_10_0.html#//apple_ref/doc/uid/TP40014305-CH11-DontLinkElementID_28)                   |


## use

### 启动
使用本地driver

```rust
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
    .build();

let d = Driver::new(option).unwrap();
d.get("https://github.com").unwrap();
```

使用远程server
```rust
let option = FirefoxBuilder::new().host("127.0.0.1").port(3824).build();
let d = Driver::new(option).unwrap();
d.get("https://github.com").unwrap();
```

### 查找元素
```rust
    driver.find_element(By::Css("#id"));
```

### 键盘操作

普通的输入，比如input输入可以
```rust
driver.find_element(By::Css("#input")).unwrap().send_keys("input").unwrap()
```

更复杂的操作，比如组合键
```rust
driver.actions()
      .key_down_special(Key::CONTROL)
      .key_down("e")
      .key_up("e")
      .key_up_special(Key::CONTROL)
      .perform()
      .unwrap();
```
