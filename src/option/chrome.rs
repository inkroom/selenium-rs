use std::{borrow::Cow, collections::HashMap, fmt::Display};

use serde::{ser::SerializeMap, Serialize};

use crate::option::MultipleTypeMapValue;

use super::{Browser, BrowserOption, Proxy};

browser_option!(
    2,
    ChromeBuilder,
    Browser::Chrome,
    pub struct ChromeOption {}
);

impl<'a> ChromeBuilder<'a> {
    ///
    /// 设置为headless模式
    ///
    pub fn head_less(self) -> Self {
        self.add_argument("--headless=new")
    }
    ///
    /// 设置为隐私模式
    ///
    pub fn private(self) -> Self {
        self.add_argument("--incognito")
    }
}
impl<'a> Serialize for ChromeOption<'a> {
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
                        .map(|f| MultipleTypeMapValue::String(Cow::from(f.as_str())))
                        .collect(),
                ),
            );
        }
        if let Some(v) = &self.binary {
            option.insert(
                "binary",
                MultipleTypeMapValue::String(Cow::from(v.as_str())),
            );
        }

        s.serialize_entry("goog:chromeOptions", &option)?;

        if !self.env.is_empty() {
            s.serialize_entry("env", &self.env)?;
        }

        s.end()
    }
}
