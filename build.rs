fn main() -> Result<(), tinyget::Error> {
    println!("rlib");

    // 从 github 上下载源码，因为实在找不到源码，只能下载构建完的js

    let is_displayed_out = format!("{}/is_displayed.rs", std::env::var("OUT_DIR").unwrap());
    if std::fs::exists(is_displayed_out.as_str()).unwrap() {
        return Ok(());
    }
    println!("file postion : {is_displayed_out}");

    let bytes = if let Ok(f) = std::env::var("MIRROR_JS_FILE") {
        std::fs::read_to_string(f)?
    } else {
        tinyget::get("https://github.com/SeleniumHQ/selenium/raw/cc5ca35d366268db87f1e510c3813114471740db/rb/lib/selenium/webdriver/atoms/isDisplayed.js")
            .send().expect("download from github fail").as_str().expect("download from github fail").to_string()
    };

    std::fs::write(
        is_displayed_out,
        format!(
            r####"pub const IS_DISPLAY_SCRIPT:&str=r###"{}"###;"####,
            bytes
        ),
    )
    .unwrap();

    Ok(())
}
