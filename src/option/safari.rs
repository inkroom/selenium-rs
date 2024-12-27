use std::{clone, collections::HashMap, fmt::Display};

use serde::{ser::SerializeMap, Serialize};

use crate::option::MultipleTypeMapValue;

use super::{Browser, BrowserOption, Proxy};

browser_option!(
    SafariBuilder,
    Browser::Safari,
    pub struct SafariOption {}
);

impl SafariBuilder {
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
impl Serialize for SafariOption {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_map(None)?;

        s.serialize_entry("browserName", "Safari")?;

        if let Some(proxy) = &self.proxy {
            s.serialize_entry("proxy", proxy)?;
        }

        s.end()
    }
}

impl Display for SafariOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}",
            serde_json::to_string(self).map_err(|f| std::fmt::Error)?
        ))
    }
}

// #[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::option::{SafariOption, MultipleTypeMapValue};

    #[test]
    fn serde() {
        let f = SafariOption {
            url: None,
            driver: None,
            binary: Some("3".to_string()),
            arguments: vec!["1".to_string(), "2".to_string()],
            exec: None,
            env: HashMap::new(),
            pref: HashMap::new(),
            proxy: None,
        };
        println!("{}", f);
        assert_eq!(
            r#"{"browserName":"Safari"}"#,
            serde_json::to_string(&f).unwrap()
        );
    }
}
