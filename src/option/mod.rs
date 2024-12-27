use std::{collections::HashMap, fmt::Display};

use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize,
};

#[macro_export]
macro_rules! browser_option{
    (
        $builder_name:ident,
        $browser:expr,
     // meta data about struct
     $(#[$meta:meta])*
     $vis:vis struct $struct_name:ident {
        $(
        // meta data about field
        $(#[$field_meta:meta])*
        $field_vis:vis $field_name:ident : $field_type:ty
        ),*$(,)?
    }
    ) => {

            $(#[$meta])*
            $vis struct $struct_name{
                pub(crate) url: Option<String>,
                pub(crate) driver: Option<String>,
                pub(crate) binary: Option<String>,
                pub(crate) arguments: Vec<String>,
                pub(crate) exec: Option<String>,
                pub(crate) env: std::collections::HashMap<String, String>,
                pub(crate) pref: std::collections::HashMap<String,  $crate::option::MultipleTypeMapValue>,
                pub(crate) proxy: Option<Proxy>,
                $(
                    $(#[$field_meta])*
                    $field_vis $field_name : $field_type,
                )*
            }

            $vis struct $builder_name{
                pub(crate) url: Option<String>,
                pub(crate) driver: Option<String>,
                pub(crate) binary: Option<String>,
                pub(crate) arguments: Vec<String>,
                pub(crate) exec: Option<String>,
                pub(crate) env: std::collections::HashMap<String, String>,
                pub(crate) pref: std::collections::HashMap<String, $crate::option::MultipleTypeMapValue>,
                pub(crate) proxy:Option<$crate::option::Proxy>,
                $(
                    $(#[$field_meta])*
                    $field_vis $field_name : $field_type,
                )*
            }

            impl BrowserOption for $struct_name {
                fn url(&self) -> Option<&str> {
                    self.url.as_deref()
                }

                fn driver(&self) -> Option<&str> {
                    self.driver.as_deref()
                }

                fn arguments(&self) -> &Vec<String> {
                    &self.arguments
                }

                fn execute(&self) -> Option<&str> {
                    self.exec.as_deref()
                }

                fn env(&self) -> &std::collections::HashMap<std::string::String, std::string::String> {
                    &self.env
                }

                fn proxy(&self) -> Option<&$crate::option::Proxy> {
                    self.proxy.as_ref()
                }

                fn browser(&self)->$crate::option::Browser{
                    $browser
                }
            }

            impl $builder_name {
                pub fn url(mut self, url: &str) -> Self {
                    self.url = Some(url.to_string());
                    self
                }

                pub fn driver(mut self, path: &str) -> Self {
                    self.driver = Some(path.to_string());
                    self
                }

                pub fn binary(mut self, binary: &str) -> Self {
                    self.binary = Some(binary.to_string());
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

                pub fn proxy(mut self, proxy:Proxy) -> Self {
                    self.proxy = Some(proxy);
                    self
                }


                pub fn new() -> Self {
                    Self {
                        url: None,
                        driver: None,
                        binary: None,
                        arguments: Vec::new(),
                        exec: None,
                        env: std::collections::HashMap::new(),
                        pref: std::collections::HashMap::new(),
                        proxy:None,
                    }
                }

                pub fn add_env(mut self, key: &str, value: &str) -> Self {
                    self.env.insert(key.to_string(), value.to_string());
                    self
                }

                pub fn add_pref_i32(mut self, key: &str, value: i32) -> Self {
                    self.pref
                        .insert(key.to_string(), $crate::option::MultipleTypeMapValue::Number(value));
                    self
                }

                pub fn add_pref_string(mut self, key: &str, value: &str) -> Self {
                    self.pref.insert(
                        key.to_string(),
                        $crate::option::MultipleTypeMapValue::String(value.to_string()),
                    );
                    self
                }

                pub fn build(self) -> impl BrowserOption {
                    $struct_name {
                        url: self.url,
                        driver: self.driver,
                        arguments: self.arguments,
                        binary: self.binary,
                        exec: self.exec,
                        env: self.env,
                        pref: self.pref,
                        proxy:self.proxy,
                    }
                }
            }


    }
}
#[derive(Clone)]
pub enum Browser {
    Firefox,
    Chrome,
    Safari,
}

impl Display for Browser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Browser::Firefox => f.write_str("firefox"),
            Browser::Chrome => f.write_str("chrome"),
            Browser::Safari => f.write_str("safari"),
        }
    }
}

#[derive(Clone)]
pub(crate) enum MultipleTypeMapValue {
    Number(i32),
    String(String),
    Map(HashMap<String, MultipleTypeMapValue>),
    Array(Vec<MultipleTypeMapValue>),
}

impl Serialize for MultipleTypeMapValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            MultipleTypeMapValue::Number(v) => serializer.serialize_i32(*v),
            MultipleTypeMapValue::String(v) => serializer.serialize_str(v.as_str()),
            MultipleTypeMapValue::Map(hash_map) => {
                let mut s = serializer.serialize_map(Some(hash_map.len()))?;
                for (key, value) in hash_map.iter() {
                    s.serialize_entry(key.as_str(), value)?;
                }
                s.end()
            }
            MultipleTypeMapValue::Array(vec) => {
                let mut s = serializer.serialize_seq(Some(vec.len()))?;
                for ele in vec.iter() {
                    s.serialize_element(ele)?;
                }
                s.end()
            }
        }
    }
}

pub trait BrowserOption: Display {
    ///
    /// 如果需要支持https需要开启https features
    ///
    fn url(&self) -> Option<&str>;
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
    /// proxy
    fn proxy(&self) -> Option<&Proxy>;

    fn browser(&self) -> Browser;
}
pub enum ProxyType {
    /// Proxy auto-configuration from URL
    Pac,
    /// Direct connection, no proxy (default on Windows)
    Direct,
    /// Proxy auto-detection (presumably with WPAD)
    AutoDetect,
    /// Use system settings (default on Linux)
    System,
    /// Manual proxy settings (e.g. for httpProxy)
    Manual,
}
pub struct Proxy {
    proxy_type: ProxyType,
    /// Defines the URL for a proxy autoconfiguration file if proxyType is equal to "pac".
    proxy_autoconfig_url: Option<String>,
    /// Defines the proxy host for FTP traffic when the proxyType is "manual".
    ///
    /// A host and optional port for scheme "ftp".
    ftp_proxy: Option<String>,

    /// Defines the proxy host for HTTP traffic when the proxyType is "manual".
    /// A host and optional port for scheme "http".
    http_proxy: Option<String>,
    /// Lists the address for which the proxy should be bypassed when the proxyType is "manual".
    /// A List containing any number of Strings.
    no_proxy: Vec<String>,
    /// Defines the proxy host for encrypted TLS traffic when the proxyType is "manual".
    /// A host and optional port for scheme "https".
    ssl_proxy: Option<String>,
    /// Defines the proxy host for a SOCKS proxy when the proxyType is "manual".
    ///
    /// A host and optional port with an undefined scheme.
    socks_proxy: Option<String>,
    /// Defines the SOCKS proxy version when the proxyType is "manual".
    ///
    /// Any integer between 0 and 255 inclusive.
    socks_version: Option<u8>,
}

impl Serialize for Proxy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_map(None)?;
        match self.proxy_type {
            ProxyType::Pac => {
                s.serialize_entry("proxyType", "pac")?;
                if let Some(v) = &self.proxy_autoconfig_url {
                    s.serialize_entry("proxyAutoconfigUrl", v)?;
                }
            }
            ProxyType::Direct => {
                s.serialize_entry("proxyType", "direct")?;
            }
            ProxyType::AutoDetect => {
                s.serialize_entry("proxyType", "autodetect")?;
            }
            ProxyType::System => {
                s.serialize_entry("proxyType", "system")?;
            }
            ProxyType::Manual => {
                s.serialize_entry("proxyType", "manual")?;

                if let Some(v) = &self.ftp_proxy {
                    s.serialize_entry("ftpProxy", v)?;
                }
                if let Some(v) = &self.http_proxy {
                    s.serialize_entry("httpProxy", v)?;
                }
                if let Some(v) = &self.ssl_proxy {
                    s.serialize_entry("sslProxy", v)?;
                }
                if let Some(v) = &self.socks_proxy {
                    s.serialize_entry("socksProxy", v)?;
                }
                if !self.no_proxy.is_empty() {
                    s.serialize_entry("noProxy", &self.no_proxy)?;
                }
                if let Some(v) = self.socks_version {
                    s.serialize_entry("socksVersion", &v)?;
                }
            }
        }
        s.end()
    }
}

impl Proxy {
    pub fn system() -> Self {
        Self {
            proxy_type: ProxyType::System,
            proxy_autoconfig_url: None,
            ftp_proxy: None,
            http_proxy: None,
            no_proxy: Vec::new(),
            ssl_proxy: None,
            socks_proxy: None,
            socks_version: None,
        }
    }

    pub fn auto_detect() -> Self {
        Self {
            proxy_type: ProxyType::AutoDetect,
            proxy_autoconfig_url: None,
            ftp_proxy: None,
            http_proxy: None,
            no_proxy: Vec::new(),
            ssl_proxy: None,
            socks_proxy: None,
            socks_version: None,
        }
    }
    pub fn detect() -> Self {
        Self {
            proxy_type: ProxyType::Direct,
            proxy_autoconfig_url: None,
            ftp_proxy: None,
            http_proxy: None,
            no_proxy: Vec::new(),
            ssl_proxy: None,
            socks_proxy: None,
            socks_version: None,
        }
    }
    pub fn pac(url: &str) -> Self {
        Self {
            proxy_type: ProxyType::Pac,
            proxy_autoconfig_url: Some(url.to_string()),
            ftp_proxy: None,
            http_proxy: None,
            no_proxy: Vec::new(),
            ssl_proxy: None,
            socks_proxy: None,
            socks_version: None,
        }
    }

    pub fn manual() -> Self {
        Self {
            proxy_type: ProxyType::Manual,
            proxy_autoconfig_url: None,
            ftp_proxy: None,
            http_proxy: None,
            no_proxy: Vec::new(),
            ssl_proxy: None,
            socks_proxy: None,
            socks_version: None,
        }
    }

    pub fn proxy_autoconfig_url(mut self, url: &str) -> Self {
        if let ProxyType::Pac = self.proxy_type {
            if let Some(v) = &mut self.proxy_autoconfig_url {
                v.clear();
                v.push_str(url);
            } else {
                self.proxy_autoconfig_url = Some(url.to_string());
            }
        }
        self
    }

    pub fn ftp_proxy(mut self, ftp_proxy: &str) -> Self {
        if let ProxyType::Manual = self.proxy_type {
            if let Some(v) = &mut self.ftp_proxy {
                v.clear();
                v.push_str(ftp_proxy);
            } else {
                self.ftp_proxy = Some(ftp_proxy.to_string());
            }
        }
        self
    }

    pub fn http_proxy(mut self, http_proxy: &str) -> Self {
        if let ProxyType::Manual = self.proxy_type {
            if let Some(v) = &mut self.http_proxy {
                v.clear();
                v.push_str(http_proxy);
            } else {
                self.http_proxy = Some(http_proxy.to_string());
            }
        }
        self
    }

    pub fn ssl_proxy(mut self, ssl_proxy: &str) -> Self {
        if let ProxyType::Manual = self.proxy_type {
            if let Some(v) = &mut self.ssl_proxy {
                v.clear();
                v.push_str(ssl_proxy);
            } else {
                self.ssl_proxy = Some(ssl_proxy.to_string());
            }
        }
        self
    }

    pub fn socks_proxy(mut self, socks_proxy: &str) -> Self {
        if let ProxyType::Manual = self.proxy_type {
            if let Some(v) = &mut self.socks_proxy {
                v.clear();
                v.push_str(socks_proxy);
            } else {
                self.socks_proxy = Some(socks_proxy.to_string());
            }
        }
        self
    }

    pub fn socks_version(mut self, socks_version: u8) -> Self {
        if let ProxyType::Manual = self.proxy_type {
            self.socks_version = Some(socks_version);
        }
        self
    }

    pub fn no_proxy(mut self, mut no_proxy: Vec<String>) -> Self {
        if let ProxyType::Manual = self.proxy_type {
            self.no_proxy.append(&mut no_proxy);
        }
        self
    }
}

mod chrome;
mod firefox;
mod safari;

pub use firefox::FirefoxBuilder;
pub use firefox::FirefoxOption;

pub use chrome::ChromeBuilder;
pub use chrome::ChromeOption;

pub use safari::SafariBuilder;
pub use safari::SafariOption;
