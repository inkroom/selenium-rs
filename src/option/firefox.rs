use std::{clone, collections::HashMap, fmt::Display};

use serde::{ser::SerializeMap, Serialize};

use crate::option::MultipleTypeMapValue;

use super::{Browser, BrowserOption, Proxy};

browser_option!(
    FirefoxBuilder,
    Browser::Firefox,
    pub struct FirefoxOption {}
);

impl FirefoxBuilder {
    ///
    /// 设置为headless模式
    ///
    pub fn head_leass(self) -> Self {
        self.add_argument("-headless")
    }
    ///
    /// 设置为隐私模式
    ///
    pub fn private(self) -> Self {
        self.add_argument("--private-window")
    }
}
impl Serialize for FirefoxOption {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_map(None)?;

        s.serialize_entry("browserName", "firefox")?;

        let mut option = std::collections::BTreeMap::new();
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
        s.serialize_entry("moz:firefoxOptions", &option)?;

        if let Some(proxy) = &self.proxy {
            s.serialize_entry("proxy", proxy)?;
        }

        if !self.env.is_empty() {
            s.serialize_entry("env", &self.env)?;
        }

        s.end()
    }
}

impl Display for FirefoxOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}",
            serde_json::to_string(self).map_err(|f| std::fmt::Error)?
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::option::{firefox::FirefoxOption, MultipleTypeMapValue};

    #[test]
    fn serde() {
        let f = FirefoxOption {
            url: None,
            driver: None,
            binary: Some("3".to_string()),
            arguments: vec!["1".to_string(), "2".to_string()],
            exec: None,
            env: HashMap::new(),
            pref: HashMap::from([(
                "dom.ipc.processCount".to_string(),
                MultipleTypeMapValue::Number(4),
            )]),
            proxy: None,
        };
        println!("{}", f);
        assert_eq!(
            r#"{"browserName":"firefox","moz:firefoxOptions":{"args":["1","2"],"binary":"3","prefs":{"dom.ipc.processCount":4}}}"#,
            serde_json::to_string(&f).unwrap()
        );
    }
}
