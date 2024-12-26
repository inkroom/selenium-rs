use std::{collections::HashMap, fmt::Display};

use serde::{ser::SerializeMap, Serialize};

use crate::option::MultipleTypeMapValue;

use super::{Browser, BrowserOption, Proxy};

browser_option!(ChromeBuilder, Browser::Chrome, pub struct ChromeOption {});

impl ChromeBuilder {
    ///
    /// 设置为headless模式
    ///
    pub fn head_leass(self) -> Self {
        self.add_argument("--headless=new")
    }
    ///
    /// 设置为隐私模式
    ///
    pub fn private(self) -> Self {
        self.add_argument("--private-window")
    }
}
impl Serialize for ChromeOption {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_map(None)?;

        s.serialize_entry("browserName", "chrome")?;

        let mut option = HashMap::new();
        if !self.pref.is_empty() {
            option.insert("prefs", MultipleTypeMapValue::Map(self.pref.clone()));
        }
        if !self.arguments.is_empty() {
            option.insert(
                "args",
                MultipleTypeMapValue::Array(
                    self.arguments
                        .iter()
                        .map(|f| MultipleTypeMapValue::String(f.clone()))
                        .collect(),
                ),
            );
        }
        if let Some(v) = &self.binary {
            option.insert("binary", MultipleTypeMapValue::String(v.clone()));
        }

        s.serialize_entry("goog:chromeOptions", &option)?;

        if !self.env.is_empty() {
            s.serialize_entry("env", &self.env)?;
        }

        s.end()
    }
}

impl Display for ChromeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}",
            serde_json::to_string(self).map_err(|f| std::fmt::Error)?
        ))
    }
}
