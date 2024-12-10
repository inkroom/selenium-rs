use std::{collections::HashMap, fmt::Display};

use serde::{
    ser::{Error, SerializeMap, SerializeSeq, SerializeStruct},
    Serialize,
};

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
    pub(crate) host: Option<String>,
    pub(crate) port: Option<u32>,
    pub(crate) driver: Option<String>,
    pub(crate) arguments: Vec<String>,
    pub(crate) exec: Option<String>,
    pub(crate) env: HashMap<String, String>,
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

    pub fn build(&self) -> impl BrowserOption {
        FirefoxOption {
            host: self.host.clone(),
            port: self.port.clone(),
            driver: self.driver.clone(),
            arguments: self.arguments.clone(),
            exec: self.exec.clone(),
            env: self.env.clone(),
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
        f.write_str(format!(r#"{{"browserName": "firefox","moz:firefoxOptions":{{"prefs": {{ "dom.ipc.processCount": 4 }},"args":[{}] }}}}"#
                ,self.arguments.iter().map(|f|format!("\"{f}\"")).collect::<Vec<String>>().join(",")
            ).as_str()
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

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
        };
        println!("{f}");
        assert_eq!(
            r#"{"browserName": "firefox","moz:firefoxOptions":{"prefs": { "dom.ipc.processCount": 4 },"args":["1","2"] }}"#,
            format!("{f}")
        );
    }
}
