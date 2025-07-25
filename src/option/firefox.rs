use std::{borrow::Cow, clone, collections::HashMap, fmt::Display, io::Write, time::UNIX_EPOCH};

use serde::{ser::SerializeMap, Serialize};

use crate::{http, option::MultipleTypeMapValue, SError};

use super::{Browser, BrowserOption, Proxy};

browser_option!(
    2,
    FirefoxBuilder,
    Browser::Firefox,
    pub struct FirefoxOption {
        ///
        /// profile base64
        ///
        pub(crate) profile: Option<String>,
    }
);

impl<'a> FirefoxBuilder<'a> {
    ///
    /// 设置为headless模式
    ///
    pub fn head_less(self) -> Self {
        self.add_argument("-headless")
    }
    ///
    /// 设置为隐私模式
    ///
    pub fn private(self) -> Self {
        self.add_argument("--private-window")
    }
    ///
    /// 禁用css加载
    ///
    pub fn disable_css(self) -> Self {
        self.add_pref_i32("permissions.default.stylesheet", 2)
    }

    ///
    /// 限制图片加载
    ///
    pub fn disable_image(self) -> Self {
        self.add_pref_i32("permissions.default.image", 2)
    }

    ///
    /// 禁用js
    ///
    pub fn disable_javascript(self) -> Self {
        self.add_pref_string("javascript.enabled", "false")
    }

    #[cfg(feature = "profile")]
    fn delete_lock_files(&self, dir: &str) -> Result<(), SError> {
        let _ = std::fs::remove_file(format!("{dir}/.parentlock"));
        let _ = std::fs::remove_file(format!("{dir}/parent.lock"));

        Ok(())
    }

    #[cfg(feature = "profile")]
    fn zip(
        &self,
        zip: &mut zip::ZipWriter<&mut std::io::Cursor<Vec<u8>>>,
        current: &str,
    ) -> Result<(), SError> {
        fn zip_inner(
            zip: &mut zip::ZipWriter<&mut std::io::Cursor<Vec<u8>>>,
            current: &str,
            dir: &str,
        ) -> Result<(), SError> {
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored)
                .unix_permissions(0o755);
            if let Ok(meta) = std::fs::metadata(current) {
                if meta.is_file() {
                    zip.start_file(current.replace(dir, ""), options).unwrap();
                    let mut b = std::fs::read(current)?;
                    zip.write_all(&mut b).unwrap();
                } else if meta.is_dir() {
                    let mut entries = std::fs::read_dir(current)?
                        .map(|res| res.map(|e| e.path()))
                        .collect::<Result<Vec<_>, std::io::Error>>()?;

                    for ele in entries {
                        zip_inner(zip, format!("{}", ele.display()).as_str(), dir)?;
                    }
                }
            }
            Ok(())
        }

        zip_inner(zip, current, current)
    }

    #[cfg(feature = "profile")]
    fn copy_dir_all(
        &self,
        src: impl AsRef<std::path::Path>,
        dst: impl AsRef<std::path::Path>,
    ) -> std::io::Result<()> {
        std::fs::create_dir_all(&dst)?;

        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.as_ref().join(src_path.file_name().unwrap());

            if file_type.is_dir() {
                // 如果是目录，则递归复制
                self.copy_dir_all(&src_path, &dst_path)?;
            } else if file_type.is_file() {
                // 如果是文件，则直接复制
                std::fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    pub fn set_profile(mut self, profile_dir: &str) -> Result<Self, SError> {
        #[cfg(feature = "profile")]
        {
            // 复制一份到临时目录
            let temp = format!(
                "{}/{}-selenium-rs/",
                std::env::temp_dir().display(),
                std::time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|f| f.as_millis())
                    .unwrap_or(0)
            );

            self.copy_dir_all(profile_dir, &temp)?;
            self.delete_lock_files(&temp)?;
            let cache = format!("{temp}/extensions.cache");
            if let Ok(meta) = std::fs::metadata(cache.as_str()) {
                if meta.is_dir() {
                    std::fs::remove_dir_all(cache.as_str())?;
                }
            }

            // zip
            let mut writer = std::io::Cursor::new(Vec::new());
            {
                let mut zip: zip::ZipWriter<&mut std::io::Cursor<Vec<u8>>> =
                    zip::ZipWriter::new(&mut writer);

                self.zip(&mut zip, &temp)?;
            }

            // zip 转base64
            let base64 = crate::base64::encode(writer.into_inner().as_slice());
            self.profile = Some(base64);
            Ok(self)
        }
        #[cfg(not(feature = "profile"))]
        {
            panic!("enable feature profile to use profile")
        }
    }
}
impl<'a> Serialize for FirefoxOption<'a> {
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

        if !self.env.is_empty() {
            let t = self
                .env
                .iter()
                .map(|(k, v)| {
                    (
                        k.to_string(),
                        MultipleTypeMapValue::String(Cow::from(v.as_str())),
                    )
                })
                .collect();
            option.insert("env", MultipleTypeMapValue::Map(t));
        }

        if let Some(profile) = &self.profile {
            option.insert(
                "profile",
                MultipleTypeMapValue::String(Cow::from(profile.as_str())),
            );
        }

        s.serialize_entry("moz:firefoxOptions", &option)?;

        if let Some(proxy) = &self.proxy {
            s.serialize_entry("proxy", proxy)?;
        }

        s.end()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::option::{firefox::FirefoxOption, FirefoxBuilder, MultipleTypeMapValue};

    #[test]
    fn serde() {
        let f = FirefoxOption {
            url: None,
            driver: None,
            binary: Some("3".to_string()),
            arguments: vec!["1".to_string(), "2".to_string()],
            env: HashMap::from([("1".to_string(), "2".to_string())]),
            pref: HashMap::from([(
                "dom.ipc.processCount".to_string(),
                MultipleTypeMapValue::Number(4),
            )]),
            timeout: 10,
            proxy: None,
            profile: None,
        };
        println!("{}", f);
        assert_eq!(
            r#"{"browserName":"firefox","moz:firefoxOptions":{"args":["1","2"],"binary":"3","env":{"1":"2"},"prefs":{"dom.ipc.processCount":4}}}"#,
            serde_json::to_string(&f).unwrap()
        );
    }

    #[test]
    #[cfg(feature = "profile")]
    fn profile() {
        let v = FirefoxBuilder::new()
            .set_profile(
                format!(
                    "{}/src",
                    std::env::current_dir()
                        .map_err(|f| SError::Message(f.to_string()))
                        .unwrap()
                        .display()
                )
                .as_str(),
            )
            .unwrap()
            .build();

        if let Some(s) = v.profile {
            println!("{s}");
        }
    }
}
