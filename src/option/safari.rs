use std::{clone, collections::HashMap, fmt::Display};

use serde::{ser::SerializeMap, Serialize};

use crate::option::MultipleTypeMapValue;

use super::{Browser, BrowserOption, Proxy};

browser_option!(
    1,
    SafariBuilder,
    Browser::Safari,
    pub struct SafariOption {}
);

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

// #[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::option::{MultipleTypeMapValue, SafariOption};

    #[test]
    fn serde() {
        let f = SafariOption {
            url: None,
            driver: None,
            binary: Some("3".to_string()),
            env: HashMap::new(),
            proxy: None,
            timeout: 10,
        };
        println!("{}", f);
        assert_eq!(
            r#"{"browserName":"Safari"}"#,
            serde_json::to_string(&f).unwrap()
        );
    }
}
