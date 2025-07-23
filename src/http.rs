//!
//! 负责实际的http通信
use std::{collections::HashMap, fmt::Display, ops::Deref, time::Duration};

use serde::{
    ser::{SerializeSeq, SerializeStruct},
    Deserialize, Serialize, Serializer,
};
use ureq::http::{self, Request};

use crate::{
    actions::Device,
    base64,
    driver::{By, Rect, Session, SwitchToFrame, TimeoutType},
    option::BrowserOption,
    Origin, SError, SResult,
};

#[derive(Deserialize)]
pub(crate) struct ResponseWrapper<T> {
    pub(crate) value: T,
}

impl<T> Deref for ResponseWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

// use serde_derive::{Deserialize, Serialize};
pub(crate) struct Http {
    url: String,
    timeout: u64,
    inner: ureq::Agent,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Capability<T> {
    pub(crate) browser_name: Option<String>,

    pub(crate) platform_name: Option<String>,

    pub(crate) always_match: Option<T>,

    pub(crate) first_match: Vec<T>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActionRequest<'a> {
    #[serde(serialize_with = "serialize_actions")]
    pub(crate) actions: Vec<Device<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parameters: Option<HashMap<String, String>>,
    #[serde(alias = "type")]
    pub(crate) _type: String,
    pub(crate) id: String,
}

fn serialize_actions<S>(v: &Vec<Device>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // serializer.collect_map(iter)
    let mut s = serializer.serialize_seq(Some(v.len()))?;

    for ele in v {
        match ele {
            Device::Pointer(pointer) => s.serialize_element(pointer)?,
            Device::Keyboard(keyboard) => s.serialize_element(keyboard)?,
            Device::Wheel(wheel) => s.serialize_element(wheel)?,
        }
    }
    s.end()
}

impl<T: BrowserOption> Display for Capability<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            format!(
                r#"{{"capabilities":{{"alwaysMatch":{{}},"firstMatch":[{}]}}}}"#,
                self.always_match
                    .as_ref()
                    .map_or("{}".to_string(), |f| format!("{f}"))
            )
            .as_str(),
        )
    }
}

impl Serialize for By<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("By", 2)?;
        match self {
            Self::Css(v) => {
                s.serialize_field("using", "css selector")?;
                s.serialize_field("value", v)?;
            }
            Self::LinkText(v) => {
                s.serialize_field("using", "link text")?;
                s.serialize_field("value", v)?;
            }
            Self::ParitialLinkText(v) => {
                s.serialize_field("using", "partial link text")?;
                s.serialize_field("value", v)?;
            }
            Self::TagName(v) => {
                s.serialize_field("using", "tag name")?;
                s.serialize_field("value", v)?;
            }
            Self::XPath(v) => {
                s.serialize_field("using", "xpath")?;
                s.serialize_field("value", v)?;
            }
            Self::Id(v) => {
                s.serialize_field("using", "css selector")?;
                s.serialize_field("value", format!("#{v}").as_str())?;
            }
            Self::Class(v) => {
                s.serialize_field("using", "css selector")?;
                s.serialize_field("value", format!(".{v}").as_str())?;
            }
        }

        s.end()
    }
}

mod script {
    include!(concat!(env!("OUT_DIR"), "/is_displayed.rs"));
}

enum Method {
    Get(String),
    Post(String, String),
    Delete(String),
}

impl Http {
    pub(crate) fn new(url: &str, timeout: u64) -> Self {
        Http {
            url: url.to_string(),
            timeout: timeout,
            inner: ureq::Agent::new_with_config(
                ureq::Agent::config_builder()
                    .timeout_connect(Some(Duration::from_secs(timeout)))
                    .timeout_global(Some(Duration::from_secs(timeout)))
                    .build(),
            ),
        }
    }

    fn req_without_res(&self, method: Method) -> SResult<()> {
        let mut v = match method {
            Method::Get(uri) => self.inner.get(uri).call(),
            Method::Post(url, body) => self
                .inner
                .post(url)
                .content_type("application/json")
                .send(body),
            Method::Delete(uri) => self.inner.delete(uri).call(),
        }?;
        Ok(())
    }

    fn req<T: serde::de::DeserializeOwned>(&self, method: Method) -> SResult<T> {
        match method {
            Method::Get(uri) => self.inner.get(uri).call(),
            Method::Post(url, body) => self
                .inner
                .post(url)
                .content_type("application/json")
                .send(body),
            Method::Delete(uri) => self.inner.delete(uri).call(),
        }?
        .body_mut()
        .read_json()
        .map_err(|f| SError::from(f))
    }

    pub(crate) fn new_session<T>(&self, cap: Capability<T>) -> SResult<Session>
    where
        T: BrowserOption,
    {
        let session: ResponseWrapper<Session> = self.req(Method::Post(
            format!("{}/session", self.url),
            format!("{cap}"),
        ))?;
        return Ok(Session {
            session_id: session.value.session_id.clone(),
        });
    }

    pub(crate) fn delete_session(&self, session_id: &str) -> SResult<()> {
        self.req_without_res(Method::Delete(format!(
            "{}/session/{}",
            self.url, session_id
        )))
    }

    pub(crate) fn navigate(&self, session_id: &str, url: &str) -> SResult<()> {
        self.req_without_res(Method::Post(
            format!("{}/session/{}/url", self.url, session_id),
            format!(r#"{{"url":"{url}"}}"#),
        ))
    }

    pub(crate) fn get_current_url(&self, session_id: &str) -> SResult<String> {
        let session: ResponseWrapper<String> = self.req(Method::Get(format!(
            "{}/session/{}/url",
            self.url, session_id
        )))?;
        Ok(session.value)
    }

    pub(crate) fn back(&self, session_id: &str) -> SResult<()> {
        self.req_without_res(Method::Post(
            format!("{}/session/{}/back", self.url, session_id),
            String::new(),
        ))
    }

    pub(crate) fn forward(&self, session_id: &str) -> SResult<()> {
        self.req_without_res(Method::Post(
            format!("{}/session/{}/forward", self.url, session_id),
            String::new(),
        ))
    }

    pub(crate) fn refresh(&self, session_id: &str) -> SResult<()> {
        self.req_without_res(Method::Post(
            format!("{}/session/{}/refresh", self.url, session_id),
            String::new(),
        ))
    }

    pub(crate) fn get_title(&self, session_id: &str) -> SResult<String> {
        let session: ResponseWrapper<String> = self.req(Method::Get(format!(
            "{}/session/{}/title",
            self.url, session_id
        )))?;
        Ok(session.value)
    }
}

/// Contexts
impl Http {
    pub(crate) fn get_window_handle(&self, session_id: &str) -> SResult<String> {
        let session: ResponseWrapper<String> = self.req(Method::Get(format!(
            "{}/session/{}/window",
            self.url, session_id
        )))?;
        Ok(session.value)
    }
    ///
    /// Returns window handles
    pub(crate) fn close_window(&self, session_id: &str) -> SResult<Vec<String>> {
        let session: ResponseWrapper<Vec<String>> = self.req(Method::Delete(format!(
            "{}/session/{}/window",
            self.url, session_id
        )))?;
        Ok(session.value)
    }

    pub(crate) fn switch_to_window(&self, session_id: &str, handle: &str) -> SResult<()> {
        self.req_without_res(Method::Post(
            format!("{}/session/{}/window", self.url, session_id),
            format!(r#"{{"handle":"{handle}"}}"#),
        ))
    }

    pub(crate) fn get_window_handles(&self, session_id: &str) -> SResult<Vec<String>> {
        let session: ResponseWrapper<Vec<String>> = self.req(Method::Get(format!(
            "{}/session/{}/window/handles",
            self.url, session_id
        )))?;

        Ok(session.value)
    }
    ///
    /// `type`: "tab" or "window"
    pub(crate) fn new_window(&self, session_id: &str, window_type: &str) -> SResult<String> {
        #[derive(Deserialize, Debug)]
        struct NewWindowResponse {
            handle: String,
            #[serde(alias = "type")]
            _type: String,
        }

        let session: ResponseWrapper<NewWindowResponse> = self.req(Method::Post(
            format!("{}/session/{}/window/new", self.url, session_id),
            format!(r#"{{"type":"{window_type}"}}"#),
        ))?;
        Ok(session.value.handle)
    }

    pub(crate) fn switch_to_frame(&self, session_id: &str, id: SwitchToFrame) -> SResult<()> {
        self.req_without_res(Method::Post(
            format!("{}/session/{}/frame", self.url, session_id),
            match id {
                SwitchToFrame::Null => r#"{"id":null}"#.to_string(),
                SwitchToFrame::Number(s) => format!(r#"{{"id":{s}}}"#),
                SwitchToFrame::Element(s) => format!(r#"{{"id":"{s}"}}"#),
            },
        ))
    }

    pub(crate) fn switch_to_parent_frame(&self, session_id: &str) -> SResult<()> {
        self.req_without_res(Method::Post(
            format!("{}/session/{}/frame", self.url, session_id),
            String::new(),
        ))
    }

    pub(crate) fn get_window_rect(&self, session_id: &str) -> SResult<Rect> {
        let session: ResponseWrapper<Rect> = self.req(Method::Get(format!(
            "{}/session/{}/window/rect",
            self.url, session_id
        )))?;

        Ok(session.value)
    }

    pub(crate) fn set_window_rect(&self, session_id: &str, rect: Rect) -> SResult<Rect> {
        let session: ResponseWrapper<Rect> = self.req(Method::Post(
            format!("{}/session/{}/window/rect", self.url, session_id),
            serde_json::to_string(&rect)?,
        ))?;

        Ok(session.value)
    }

    pub(crate) fn maximize_window(&self, session_id: &str) -> SResult<Rect> {
        let session: ResponseWrapper<Rect> = self.req(Method::Post(
            format!("{}/session/{}/window/maximize", self.url, session_id),
            String::new(),
        ))?;
        Ok(session.value)
    }

    pub(crate) fn minimize_window(&self, session_id: &str) -> SResult<Rect> {
        let session: ResponseWrapper<Rect> = self.req(Method::Post(
            format!("{}/session/{}/window/minimize", self.url, session_id),
            String::new(),
        ))?;
        Ok(session.value)
    }

    pub(crate) fn fullscreen_window(&self, session_id: &str) -> SResult<Rect> {
        let session: ResponseWrapper<Rect> = self.req(Method::Post(
            format!("{}/session/{}/window/fullscreen", self.url, session_id),
            String::new(),
        ))?;
        Ok(session.value)
    }

    pub(crate) fn find_element(&self, session_id: &str, by: By<'_>) -> SResult<(String, String)> {
        let res: ResponseWrapper<HashMap<String, String>> = self.req(Method::Post(
            format!("{}/session/{}/element", self.url, session_id),
            serde_json::to_string(&by)?,
        ))?;
        for ele in res.value {
            return Ok(ele);
        }
        Err(SError::Browser("element not found".to_string()))
    }

    pub(crate) fn find_elements(
        &self,
        session_id: &str,
        by: By<'_>,
    ) -> SResult<Vec<(String, String)>> {
        let res: ResponseWrapper<Vec<HashMap<String, String>>> = self.req(Method::Post(
            format!("{}/session/{}/elements", self.url, session_id),
            serde_json::to_string(&by)?,
        ))?;

        Ok(res
            .value
            .iter()
            .filter(|f| !f.is_empty())
            .map(|f| {
                let t = f.iter().next().unwrap();
                (t.0.to_string(), t.1.to_string())
            })
            .collect())
    }

    pub(crate) fn find_element_from_element(
        &self,
        session_id: &str,
        element_id: &str,
        by: By<'_>,
    ) -> SResult<(String, String)> {
        let res: ResponseWrapper<HashMap<String, String>> = self.req(Method::Post(
            format!(
                "{}/session/{}/element/{}/element",
                self.url, session_id, element_id
            ),
            serde_json::to_string(&by)?,
        ))?;
        for ele in res.value {
            return Ok(ele);
        }
        Err(SError::Browser("element not found".to_string()))
    }

    pub(crate) fn find_elements_from_element(
        &self,
        session_id: &str,
        element_id: &str,
        by: By<'_>,
    ) -> SResult<Vec<(String, String)>> {
        let res: ResponseWrapper<Vec<HashMap<String, String>>> = self.req(Method::Post(
            format!(
                "{}/session/{}/element/{}/elements",
                self.url, session_id, element_id
            ),
            serde_json::to_string(&by)?,
        ))?;

        Ok(res
            .value
            .iter()
            .filter(|f| !f.is_empty())
            .map(|f| {
                let t = f.iter().next().unwrap();
                (t.0.to_string(), t.1.to_string())
            })
            .collect())
    }

    pub(crate) fn get_active_element(&self, session_id: &str) -> SResult<(String, String)> {
        let res: ResponseWrapper<HashMap<String, String>> = self.req(Method::Get(format!(
            "{}/session/{}/element/active",
            self.url, session_id
        )))?;
        for ele in res.value {
            return Ok(ele);
        }
        Err(SError::Browser("element not found".to_string()))
    }

    pub(crate) fn get_element_shadow_root(
        &self,
        session_id: &str,
        element_id: &str,
    ) -> SResult<(String, String)> {
        let res: ResponseWrapper<HashMap<String, String>> = self.req(Method::Get(format!(
            "{}/session/{}/element/{}/shadow",
            self.url, session_id, element_id
        )))?;
        for ele in res.value {
            return Ok(ele);
        }
        Err(SError::Browser("element shadow not found".to_string()))
    }

    pub(crate) fn find_element_from_shadow_root(
        &self,
        session_id: &str,
        shadow_id: &str,
        by: By<'_>,
    ) -> SResult<(String, String)> {
        let res: ResponseWrapper<HashMap<String, String>> = self.req(Method::Post(
            format!(
                "{}/session/{}/shadow/{}/element",
                self.url, session_id, shadow_id
            ),
            serde_json::to_string(&by)?,
        ))?;
        for ele in res.value {
            return Ok(ele);
        }
        Err(SError::Browser("element not found".to_string()))
    }

    pub(crate) fn find_elements_from_shadow_root(
        &self,
        session_id: &str,
        shadow_id: &str,
        by: By<'_>,
    ) -> SResult<Vec<(String, String)>> {
        let res: ResponseWrapper<Vec<HashMap<String, String>>> = self.req(Method::Post(
            format!(
                "{}/session/{}/shadow/{}/elements",
                self.url, session_id, shadow_id
            ),
            serde_json::to_string(&by)?,
        ))?;
        Ok(res
            .value
            .iter()
            .filter(|f| !f.is_empty())
            .map(|f| {
                let t = f.iter().next().unwrap();
                (t.0.to_string(), t.1.to_string())
            })
            .collect())
    }

    pub(crate) fn is_element_selected(&self, session_id: &str, element_id: &str) -> SResult<bool> {
        let res: ResponseWrapper<bool> = self.req(Method::Get(format!(
            "{}/session/{}/element/{}/selected",
            self.url, session_id, element_id
        )))?;
        Ok(res.value)
    }

    pub(crate) fn get_element_attribute(
        &self,
        session_id: &str,
        element_id: &str,
        name: &str,
    ) -> SResult<Option<String>> {
        let res: ResponseWrapper<Option<String>> = self.req(Method::Get(format!(
            "{}/session/{}/element/{}/attribute/{}",
            self.url, session_id, element_id, name
        )))?;
        Ok(res.value)
    }

    pub(crate) fn get_element_property(
        &self,
        session_id: &str,
        element_id: &str,
        name: &str,
    ) -> SResult<Option<String>> {
        let res: ResponseWrapper<Option<String>> = self.req(Method::Get(format!(
            "{}/session/{}/element/{}/property/{}",
            self.url, session_id, element_id, name
        )))?;
        Ok(res.value)
    }

    pub(crate) fn get_element_css_value(
        &self,
        session_id: &str,
        element_id: &str,
        name: &str,
    ) -> SResult<String> {
        let res: ResponseWrapper<String> = self.req(Method::Get(format!(
            "{}/session/{}/element/{}/css/{}",
            self.url, session_id, element_id, name
        )))?;
        Ok(res.value)
    }

    pub(crate) fn get_element_text(&self, session_id: &str, element_id: &str) -> SResult<String> {
        let res: ResponseWrapper<String> = self.req(Method::Get(format!(
            "{}/session/{}/element/{}/text",
            self.url, session_id, element_id,
        )))?;

        Ok(res.value)
    }

    pub(crate) fn get_element_tag_name(
        &self,
        session_id: &str,
        element_id: &str,
    ) -> SResult<String> {
        let res: ResponseWrapper<String> = self.req(Method::Get(format!(
            "{}/session/{}/element/{}/name",
            self.url, session_id, element_id,
        )))?;
        Ok(res.value)
    }

    pub(crate) fn get_element_rect(&self, session_id: &str, element_id: &str) -> SResult<Rect> {
        let res: ResponseWrapper<Rect> = self.req(Method::Get(format!(
            "{}/session/{}/element/{}/rect",
            self.url, session_id, element_id,
        )))?;
        Ok(res.value)
    }

    pub(crate) fn is_element_enabled(&self, session_id: &str, element_id: &str) -> SResult<bool> {
        let res: ResponseWrapper<bool> = self.req(Method::Get(format!(
            "{}/session/{}/element/{}/enabled",
            self.url, session_id, element_id,
        )))?;
        Ok(res.value)
    }

    pub(crate) fn element_click(&self, session_id: &str, element_id: &str) -> SResult<()> {
        self.req_without_res(Method::Post(
            format!(
                "{}/session/{}/element/{}/click",
                self.url, session_id, element_id,
            ),
            "{}".to_string(),
        ))
    }

    pub(crate) fn element_clear(&self, session_id: &str, element_id: &str) -> SResult<()> {
        self.req_without_res(Method::Post(
            format!(
                "{}/session/{}/element/{}/clear",
                self.url, session_id, element_id,
            ),
            "{}".to_string(),
        ))
    }

    pub(crate) fn element_send_keys(
        &self,
        session_id: &str,
        element_id: &str,
        keys: &str,
    ) -> SResult<()> {
        self.req_without_res(Method::Post(
            format!(
                "{}/session/{}/element/{}/value",
                self.url, session_id, element_id,
            ),
            format!(
                r#"{{"text":"{keys}","value":[{}]}}"#,
                keys.chars()
                    .map(|f| format!(r#""{}""#, f))
                    .collect::<Vec<String>>()
                    .join(",")
            ),
        ))
    }

    pub(crate) fn get_page_source(&self, session_id: &str) -> SResult<String> {
        let res: ResponseWrapper<String> = self.req(Method::Get(format!(
            "{}/session/{}/source",
            self.url, session_id,
        )))?;
        Ok(res.value)
    }

    pub(crate) fn execute_script<T: serde::de::DeserializeOwned>(
        &self,
        session_id: &str,
        script: &str,
        args: &[&str],
    ) -> SResult<T> {
        #[derive(Serialize)]
        struct TempExecuteScript {
            script: String,
            args: Vec<String>,
        }
        let t = TempExecuteScript {
            script: script.to_string(),
            args: args.iter().map(|f| f.to_string()).collect(),
        };

        let res: ResponseWrapper<T> = self.req(Method::Post(
            format!("{}/session/{}/execute/sync", self.url, session_id),
            serde_json::to_string(&t)?,
        ))?;
        Ok(res.value)
    }

    pub(crate) fn execute_async_script<T: serde::de::DeserializeOwned>(
        &self,
        session_id: &str,
        script: &str,
        args: &[&str],
    ) -> SResult<T> {
        #[derive(Serialize)]
        struct TempExecuteScript {
            script: String,
            args: Vec<String>,
        }
        let t = TempExecuteScript {
            script: script.to_string(),
            args: args.iter().map(|f| f.to_string()).collect(),
        };

        let res: ResponseWrapper<T> = self.req(Method::Post(
            format!("{}/session/{}/execute/async", self.url, session_id),
            serde_json::to_string(&t)?,
        ))?;
        Ok(res.value)
    }

    pub(crate) fn set_timeouts(&self, session_id: &str, timeout: TimeoutType) -> SResult<()> {
        self.req_without_res(Method::Post(
            format!("{}/session/{}/timeouts", self.url, session_id),
            match timeout {
                TimeoutType::Script(t) => format!(r#"{{"script":{t}}}"#),
                TimeoutType::PageLoad(t) => format!(r#"{{"pageLoad":{t}}}"#),
                TimeoutType::Implicit(t) => format!(r#"{{"implicit":{t}}}"#),
            },
        ))
    }

    pub(crate) fn get_timouts(&self, session_id: &str) -> SResult<Vec<TimeoutType>> {
        let res: ResponseWrapper<HashMap<String, u32>> = self.req(Method::Get(format!(
            "{}/session/{}/timeouts",
            self.url, session_id
        )))?;

        Ok(res
            .value
            .iter()
            .map(|(key, value)| {
                if key == "script" {
                    TimeoutType::Script(*value)
                } else if key == "pageLoad" {
                    TimeoutType::PageLoad(*value)
                } else {
                    TimeoutType::Implicit(*value)
                }
            })
            .collect())
    }

    pub(crate) fn perform_actions(
        &self,
        session_id: &str,
        req: Vec<ActionRequest<'_>>,
    ) -> SResult<()> {
        let mut map = HashMap::new();
        map.insert("actions", req);
        let j: String = serde_json::to_string(&map)?;
        self.req_without_res(Method::Post(
            format!("{}/session/{}/actions", self.url, session_id),
            j,
        ))
    }

    pub(crate) fn dismiss_alert(&self, session_id: &str) -> SResult<()> {
        self.req_without_res(Method::Post(
            format!("{}/session/{}/alert/dismiss", self.url, session_id),
            "{}".to_string(),
        ))
    }

    pub(crate) fn accept_alert(&self, session_id: &str) -> SResult<()> {
        self.req_without_res(Method::Post(
            format!("{}/session/{}/alert/accept", self.url, session_id),
            "{}".to_string(),
        ))
    }

    pub(crate) fn get_alert_text(&self, session_id: &str) -> SResult<String> {
        let res: ResponseWrapper<String> = self.req(Method::Get(format!(
            "{}/session/{}/alert/text",
            self.url, session_id
        )))?;
        Ok(res.value)
    }

    pub(crate) fn send_alert_text(&self, session_id: &str, text: &str) -> SResult<()> {
        self.req_without_res(Method::Post(
            format!("{}/session/{}/alert/text", self.url, session_id),
            format!(r#"{{"text":"{text}"}}"#),
        ))
    }
    pub(crate) fn take_screenshot(&self, session_id: &str) -> SResult<Vec<u8>> {
        let res: ResponseWrapper<String> = self.req(Method::Get(format!(
            "{}/session/{}/screenshot",
            self.url, session_id
        )))?;
        Ok(base64::decode(res.value.as_bytes()))
    }
    pub(crate) fn take_element_screenshot(
        &self,
        session_id: &str,
        element_id: &str,
    ) -> SResult<Vec<u8>> {
        let res: ResponseWrapper<String> = self.req(Method::Get(format!(
            "{}/session/{}/element/{}/screenshot",
            self.url, session_id, element_id
        )))?;
        Ok(base64::decode(res.value.as_bytes()))
    }

    pub(crate) fn is_element_displayed(&self, session_id: &str, element: Origin) -> SResult<bool> {
        #[derive(Serialize)]
        struct TempExecuteScript {
            script: String,
            args: Vec<Origin>,
        }
        let t = TempExecuteScript {
            script: format!(
                "return ({}).apply(null, arguments);",
                script::IS_DISPLAY_SCRIPT.to_string()
            ),
            args: vec![element],
        };

        let res: ResponseWrapper<bool> = self.req(Method::Post(
            format!("{}/session/{}/execute/sync", self.url, session_id),
            serde_json::to_string(&t)?,
        ))?;
        Ok(res.value)
    }
}

impl From<ureq::Error> for SError {
    fn from(value: ureq::Error) -> Self {
        match value {
            ureq::Error::StatusCode(code) => SError::Http(code as i32, "".to_string()),
            // ureq::Error::Http(e)=>{ SError::Http(, ()) },
            _ => SError::Http(-1, format!("{}", value)),
        }
    }
}

impl From<serde_json::Error> for SError {
    fn from(value: serde_json::Error) -> Self {
        SError::Http(-2, format!("{}", value))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        driver::Rect,
        option::{FirefoxOption, MultipleTypeMapValue},
    };

    use super::Capability;

    #[test]
    fn test_capability() {
        let r = Rect {
            x: None,
            y: Some(32.0),
            width: None,
            height: Some(39.0),
        };
        println!("{}", serde_json::to_string(&r).unwrap());
        let c = Capability {
            browser_name: None,
            platform_name: None,
            always_match: Some(FirefoxOption {
                url: None,
                driver: None,
                arguments: vec!["1".to_string(), "2".to_string()],
                env: HashMap::new(),
                pref: HashMap::from([(
                    "dom.ipc.processCount".to_string(),
                    MultipleTypeMapValue::Number(4),
                )]),
                timeout: 10,
                proxy: None,
                binary: None,
                profile: None,
            }),
            first_match: Vec::new(),
        };

        println!("{c}");
        assert_eq!(
            r#"{"capabilities":{"alwaysMatch":{},"firstMatch":[{"browserName":"firefox","moz:firefoxOptions":{"args":["1","2"],"prefs":{"dom.ipc.processCount":4}}}]}}"#,
            format!("{c}")
        );

        let c: Capability<FirefoxOption> = Capability {
            browser_name: None,
            platform_name: None,
            always_match: None,
            first_match: Vec::new(),
        };

        println!("{c}");
        assert_eq!(
            r#"{"capabilities":{"alwaysMatch":{},"firstMatch":[{}]}}"#,
            format!("{c}")
        );
    }
}
