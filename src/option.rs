use std::{collections::HashMap, fmt::Display};

use serde::{
    ser::{Error, SerializeMap, SerializeSeq, SerializeStruct},
    Serialize,
};

pub(crate) enum MultipleTypeMapValue {
    i32(i32),
    String(String),
}

pub trait BrowserOption: Display {
    ///
    ///
    fn host(&self) -> Option<&str>;

    fn port(&self) -> Option<u32>;
    ///
    /// driver file path
    ///
    fn driver(&self) -> Option<&str>;

    fn arguments(&self) -> &Vec<String>;
    ///
    /// 浏览器可执行文件位置
    ///
    fn execute(&self) -> Option<&str>;

    fn env(&self) -> &HashMap<std::string::String, std::string::String>;
}

pub struct FirefoxBuilder {
    host: Option<String>,
    port: Option<u32>,
    driver: Option<String>,
    arguments: Vec<String>,
    exec: Option<String>,
    env: HashMap<String, String>,
    pref: HashMap<String, MultipleTypeMapValue>,
}

impl FirefoxBuilder {
    pub fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }

    pub fn port(mut self, port: u32) -> Self {
        self.port = Some(port);
        self
    }

    pub fn driver(mut self, path: &str) -> Self {
        self.driver = Some(path.to_string());
        self
    }

    pub fn add_argument(mut self, arg: &str) -> Self {
        self.arguments.push(arg.to_string());
        self
    }

    pub fn execute(mut self, path: &str) -> Self {
        self.exec = Some(path.to_string());
        self
    }

    pub fn new() -> Self {
        FirefoxBuilder {
            host: None,
            port: None,
            driver: None,
            arguments: Vec::new(),
            exec: None,
            env: HashMap::new(),
            pref: HashMap::from([(
                "dom.ipc.processCount".to_string(),
                MultipleTypeMapValue::i32(3),
            )]),
        }
    }
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

    pub fn add_env(mut self, key: &str, value: &str) -> Self {
        self.env.insert(key.to_string(), value.to_string());
        self
    }

    pub fn add_pref_i32(mut self, key: &str, value: i32) -> Self {
        self.pref
            .insert(key.to_string(), MultipleTypeMapValue::i32(value));
        self
    }

    pub fn add_pref_string(mut self, key: &str, value: &str) -> Self {
        self.pref.insert(
            key.to_string(),
            MultipleTypeMapValue::String(value.to_string()),
        );
        self
    }

    pub fn build(self) -> impl BrowserOption {
        FirefoxOption {
            host: self.host,
            port: self.port,
            driver: self.driver,
            arguments: self.arguments,
            exec: self.exec,
            env: self.env,
            pref: self.pref,
        }
    }
}
pub(crate) struct FirefoxOption {
    pub(crate) host: Option<String>,
    pub(crate) port: Option<u32>,
    pub(crate) driver: Option<String>,
    pub(crate) arguments: Vec<String>,
    pub(crate) exec: Option<String>,
    pub(crate) env: HashMap<String, String>,
    pub(crate) pref: HashMap<String, MultipleTypeMapValue>,
}

impl BrowserOption for FirefoxOption {
    fn host(&self) -> Option<&str> {
        self.host.as_ref().map(|x| x.as_str())
    }

    fn port(&self) -> Option<u32> {
        self.port
    }

    fn driver(&self) -> Option<&str> {
        self.driver.as_ref().map(|x| x.as_str())
    }

    fn arguments(&self) -> &Vec<String> {
        &self.arguments
    }

    fn execute(&self) -> Option<&str> {
        self.exec.as_ref().map(|x| x.as_str())
    }

    fn env(&self) -> &HashMap<std::string::String, std::string::String> {
        &self.env
    }
}

impl Display for FirefoxOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!(r#"{{"browserName": "firefox","moz:firefoxOptions":{{"prefs": {{ {} }},"args":[{}] }}}}"#
        ,self.pref.iter().map(|(key,value)|{
            format!(r#""{key}": {}"#,match value {
                MultipleTypeMapValue::i32(v) => v.to_string(),
                MultipleTypeMapValue::String(v) => format!(r#""{v}""#),
            })
        }).collect::<Vec<String>>().join(",")
                ,self.arguments.iter().map(|f|format!("\"{f}\"")).collect::<Vec<String>>().join(",")
            ).as_str()
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::option::MultipleTypeMapValue;

    use super::FirefoxOption;

    #[test]
    fn test_seq() {
        let f = FirefoxOption {
            host: None,
            port: None,
            driver: None,
            arguments: vec!["1".to_string(), "2".to_string()],
            exec: None,
            env: HashMap::new(),
            pref: HashMap::from([(
                "dom.ipc.processCount".to_string(),
                MultipleTypeMapValue::i32(4),
            )]),
        };
        println!("{f}");
        assert_eq!(
            r#"{"browserName": "firefox","moz:firefoxOptions":{"prefs": { "dom.ipc.processCount": 4 },"args":["1","2"] }}"#,
            format!("{f}")
        );
    }
}
