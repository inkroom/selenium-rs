use std::{borrow::Cow, fmt::Display};

use serde::{ser::SerializeMap, Serialize};

use super::{Browser, MultipleTypeMapValue};

browser_option!(2, EdgeBuilder, Browser::Edge, pub struct EdgeOption {});

impl <'a> Serialize for EdgeOption<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_map(None)?;
        s.serialize_entry("browserName", "MicrosoftEdge")?;

        let mut option = std::collections::BTreeMap::new();
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
        s.serialize_entry("ms:edgeOptions", &option)?;
        if let Some(proxy) = &self.proxy {
            s.serialize_entry("proxy", proxy)?;
        }
        s.end()
        // {"browserName":"MicrosoftEdge","ms:edgeOptions":{"args":["headless"],"extensions":[]},"proxy":{"autodetect":false,"httpProxy":"http://127.0.0.1:1254","proxyType":1}}
    }
}
impl<'a> EdgeBuilder<'a> {
    ///
    /// 设置为headless模式
    ///
    pub fn head_less(self) -> Self {
        self.add_argument("headless")
    }
    ///
    /// 设置为隐私模式
    ///
    pub fn private(self) -> Self {
        self.add_argument("-inprivate")
    }
    
}
