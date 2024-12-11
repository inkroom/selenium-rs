//!
//! 负责实际的http通信
use std::{collections::HashMap, fmt::Display, ops::Deref};

use serde::{
    ser::{SerializeSeq, SerializeStruct},
    Deserialize, Serialize, Serializer,
};

use crate::{
    actions::Device,
    driver::{By, Rect, Session, SwitchToFrame, TimeoutType},
    element::Element,
    option::BrowserOption,
    SError, SResult,
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
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Capability<T> {
    pub(crate) browser_name: Option<String>,

    pub(crate) platform_name: Option<String>,

    pub(crate) always_match: Option<T>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActionRequest<'a> {
    #[serde(serialize_with = "serialize_actions")]
    pub(crate) actions: Vec<Device<'a>>,
    pub(crate) parameters: HashMap<String, String>,
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
        }
    }
    s.end()
}

impl<T: BrowserOption> Display for Capability<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            format!(
                r#"{{"capabilities":{{"alwaysMatch":{}}}}}"#,
                self.always_match
                    .as_ref()
                    .map_or("{}".to_string(), |f| format!("{f}"))
            )
            .as_str(),
        )
    }
}

impl<'a> Serialize for By<'a> {
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
        }

        s.end()
    }
}

impl Http {
    pub(crate) fn new(url: &str) -> Self {
        Http {
            url: url.to_string(),
        }
    }

    pub(crate) fn new_session<T>(&self, cap: Capability<T>) -> SResult<Session>
    where
        T: BrowserOption,
    {
        let v = minreq::post(format!("{}/session", self.url))
            .with_body(format!("{cap}"))
            .with_header("Content-Type", "application/json")
            .send()?;

        if v.status_code == 200 {
            let session: ResponseWrapper<Session> = serde_json::from_str(v.as_str()?)?;
            return Ok(Session {
                session_id: session.value.session_id.clone(),
            });
        }
        Err(SError::message(format!(
            "status={} {}",
            v.status_code,
            v.as_str()?
        )))
    }

    pub(crate) fn delete_session(&self, session_id: &str) -> SResult<()> {
        let _v = minreq::delete(format!("{}/session/{}", self.url, session_id)).send()?;
        Ok(())
    }

    pub(crate) fn navigate(&self, session_id: &str, url: &str) -> SResult<()> {
        let _v = minreq::post(format!("{}/session/{}/url", self.url, session_id))
            .with_header("Content-Type", "application/json")
            .with_body(format!(r#"{{"url":"{url}"}}"#))
            .send()?;

        Ok(())
    }

    pub(crate) fn get_current_url(&self, session_id: &str) -> SResult<String> {
        let v = minreq::get(format!("{}/session/{}/url", self.url, session_id))
            .with_header("Content-Type", "application/json")
            .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        let session: ResponseWrapper<String> = serde_json::from_str(v.as_str()?)?;
        return Ok(session.value.clone());
    }

    pub(crate) fn back(&self, session_id: &str) -> SResult<()> {
        let v = minreq::post(format!("{}/session/{}/back", self.url, session_id))
            .with_header("Content-Type", "application/json")
            .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        Ok(())
    }

    pub(crate) fn forward(&self, session_id: &str) -> SResult<()> {
        let v = minreq::post(format!("{}/session/{}/forward", self.url, session_id))
            .with_header("Content-Type", "application/json")
            .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        Ok(())
    }

    pub(crate) fn refresh(&self, session_id: &str) -> SResult<()> {
        let v = minreq::post(format!("{}/session/{}/refresh", self.url, session_id))
            .with_header("Content-Type", "application/json")
            .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        Ok(())
    }

    pub(crate) fn get_title(&self, session_id: &str) -> SResult<String> {
        let v = minreq::get(format!("{}/session/{}/title", self.url, session_id))
            .with_header("Content-Type", "application/json")
            .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        let session: ResponseWrapper<String> = serde_json::from_str(v.as_str()?)?;
        return Ok(session.value.clone());
    }
}

/// Contexts
impl Http {
    pub(crate) fn get_window_handle(&self, session_id: &str) -> SResult<String> {
        let v = minreq::get(format!("{}/session/{}/window", self.url, session_id))
            .with_header("Content-Type", "application/json")
            .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        let session: ResponseWrapper<String> = serde_json::from_str(v.as_str()?)?;
        return Ok(session.value.clone());
    }
    ///
    /// Returns window handles
    pub(crate) fn close_window(&self, session_id: &str) -> SResult<Vec<String>> {
        let v = minreq::delete(format!("{}/session/{}/window", self.url, session_id))
            .with_header("Content-Type", "application/json")
            .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        let session: ResponseWrapper<Vec<String>> = serde_json::from_str(v.as_str()?)?;
        return Ok(session.value.clone());
    }

    pub(crate) fn switch_to_window(&self, session_id: &str, handle: &str) -> SResult<()> {
        let v = minreq::delete(format!("{}/session/{}/window", self.url, session_id))
            .with_header("Content-Type", "application/json")
            .with_body(format!(r#"{{"handle":"{handle}","name":"{handle}"}}"#))
            .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        return Ok(());
    }

    pub(crate) fn get_window_handles(&self, session_id: &str) -> SResult<Vec<String>> {
        let v = minreq::delete(format!(
            "{}/session/{}/window/handles",
            self.url, session_id
        ))
        .with_header("Content-Type", "application/json")
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        let session: ResponseWrapper<Vec<String>> = serde_json::from_str(v.as_str()?)?;
        return Ok(session.value.clone());
    }
    ///
    /// `type`: "tab" or "window"
    pub(crate) fn new_window(&self, session_id: &str, window_type: &str) -> SResult<String> {
        let v = minreq::post(format!("{}/session/{}/window/new", self.url, session_id))
            .with_header("Content-Type", "application/json")
            .with_body(format!(r#"{{"type":"{window_type}"}}"#))
            .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        #[derive(Deserialize, Debug)]
        struct NewWindowResponse {
            handle: String,
            #[serde(alias = "type")]
            _type: String,
        }

        let session: ResponseWrapper<NewWindowResponse> = serde_json::from_str(v.as_str()?)?;
        return Ok(session.value.handle.clone());
    }

    pub(crate) fn switch_to_frame(&self, session_id: &str, id: SwitchToFrame) -> SResult<()> {
        let mut req = minreq::post(format!("{}/session/{}/frame", self.url, session_id))
            .with_header("Content-Type", "application/json");
        match id {
            SwitchToFrame::Null => {
                req = req.with_body(format!(r#"{{"id":null}}"#));
            }
            SwitchToFrame::Number(s) => {
                req = req.with_body(format!(r#"{{"id":{s}}}"#));
            }
            SwitchToFrame::Element(s) => {
                req = req.with_body(format!(r#"{{"id":"{s}"}}"#));
            }
        }
        let v = req.send()?;
        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        return Ok(());
    }

    pub(crate) fn switch_to_parent_frame(&self, session_id: &str) -> SResult<()> {
        let req = minreq::post(format!("{}/session/{}/frame", self.url, session_id))
            .with_header("Content-Type", "application/json");

        let v = req.send()?;
        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        return Ok(());
    }

    pub(crate) fn get_window_rect(&self, session_id: &str) -> SResult<Rect> {
        let v = minreq::get(format!("{}/session/{}/window/rect", self.url, session_id))
            .with_header("Content-Type", "application/json")
            .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        let session: ResponseWrapper<Rect> = serde_json::from_str(v.as_str()?)?;
        return Ok(session.value.clone());
    }

    pub(crate) fn set_window_rect(&self, session_id: &str, rect: Rect) -> SResult<Rect> {
        let v = minreq::post(format!("{}/session/{}/window/rect", self.url, session_id))
            .with_header("Content-Type", "application/json")
            .with_body(serde_json::to_string(&rect)?)
            .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        let session: ResponseWrapper<Rect> = serde_json::from_str(v.as_str()?)?;
        return Ok(session.value.clone());
    }

    pub(crate) fn maximize_window(&self, session_id: &str) -> SResult<Rect> {
        let v = minreq::post(format!(
            "{}/session/{}/window/maximize",
            self.url, session_id
        ))
        .with_header("Content-Type", "application/json")
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        let session: ResponseWrapper<Rect> = serde_json::from_str(v.as_str()?)?;
        return Ok(session.value.clone());
    }

    pub(crate) fn minimize_window(&self, session_id: &str) -> SResult<Rect> {
        let v = minreq::post(format!(
            "{}/session/{}/window/minimize",
            self.url, session_id
        ))
        .with_header("Content-Type", "application/json")
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        let session: ResponseWrapper<Rect> = serde_json::from_str(v.as_str()?)?;
        return Ok(session.value.clone());
    }

    pub(crate) fn fullscreen_window(&self, session_id: &str) -> SResult<Rect> {
        let v = minreq::post(format!(
            "{}/session/{}/window/fullscreen",
            self.url, session_id
        ))
        .with_header("Content-Type", "application/json")
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        let session: ResponseWrapper<Rect> = serde_json::from_str(v.as_str()?)?;
        return Ok(session.value.clone());
    }

    pub(crate) fn find_element<'a>(
        &self,
        session_id: &str,
        by: By<'a>,
    ) -> SResult<(String, String)> {
        let v = minreq::post(format!("{}/session/{}/element", self.url, session_id))
            .with_header("Content-Type", "application/json")
            .with_body(serde_json::to_string(&by)?)
            .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        let res: ResponseWrapper<HashMap<String, String>> = serde_json::from_str(v.as_str()?)?;
        for ele in res.value {
            return Ok(ele);
        }
        return Err(SError::message("element not found".to_string()));
    }

    pub(crate) fn find_elements<'a>(
        &self,
        session_id: &str,
        by: By<'a>,
    ) -> SResult<Vec<(String, String)>> {
        let v = minreq::post(format!("{}/session/{}/elements", self.url, session_id))
            .with_header("Content-Type", "application/json")
            .with_body(serde_json::to_string(&by)?)
            .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        let res: ResponseWrapper<Vec<HashMap<String, String>>> = serde_json::from_str(v.as_str()?)?;
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

    pub(crate) fn find_element_from_element<'a>(
        &self,
        session_id: &str,
        element_id: &str,
        by: By<'a>,
    ) -> SResult<(String, String)> {
        let v = minreq::post(format!(
            "{}/session/{}/element/{}/element",
            self.url, session_id, element_id
        ))
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&by)?)
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        let res: ResponseWrapper<HashMap<String, String>> = serde_json::from_str(v.as_str()?)?;
        for ele in res.value {
            return Ok(ele);
        }
        return Err(SError::message("element not found".to_string()));
    }

    pub(crate) fn find_elements_from_element<'a>(
        &self,
        session_id: &str,
        element_id: &str,
        by: By<'a>,
    ) -> SResult<Vec<(String, String)>> {
        let v = minreq::post(format!(
            "{}/session/{}/element/{}/elements",
            self.url, session_id, element_id
        ))
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&by)?)
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        let res: ResponseWrapper<Vec<HashMap<String, String>>> = serde_json::from_str(v.as_str()?)?;
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
        let v = minreq::get(format!(
            "{}/session/{}/element/active",
            self.url, session_id
        ))
        .with_header("Content-Type", "application/json")
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        let res: ResponseWrapper<HashMap<String, String>> = serde_json::from_str(v.as_str()?)?;
        for ele in res.value {
            return Ok(ele);
        }
        return Err(SError::message("element not found".to_string()));
    }

    pub(crate) fn get_element_shadow_root(
        &self,
        session_id: &str,
        element_id: &str,
    ) -> SResult<(String, String)> {
        let v = minreq::get(format!(
            "{}/session/{}/element/{}/shadow",
            self.url, session_id, element_id
        ))
        .with_header("Content-Type", "application/json")
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        let res: ResponseWrapper<HashMap<String, String>> = serde_json::from_str(v.as_str()?)?;
        for ele in res.value {
            return Ok(ele);
        }
        return Err(SError::message("element shadow not found".to_string()));
    }

    pub(crate) fn find_element_from_shadow_root<'a>(
        &self,
        session_id: &str,
        shadow_id: &str,
        by: By<'a>,
    ) -> SResult<(String, String)> {
        let v = minreq::post(format!(
            "{}/session/{}/shadow/{}/element",
            self.url, session_id, shadow_id
        ))
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&by)?)
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        let res: ResponseWrapper<HashMap<String, String>> = serde_json::from_str(v.as_str()?)?;
        for ele in res.value {
            return Ok(ele);
        }
        return Err(SError::message("element not found".to_string()));
    }

    pub(crate) fn find_elements_from_shadow_root<'a>(
        &self,
        session_id: &str,
        shadow_id: &str,
        by: By<'a>,
    ) -> SResult<Vec<(String, String)>> {
        let v = minreq::post(format!(
            "{}/session/{}/shadow/{}/elements",
            self.url, session_id, shadow_id
        ))
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&by)?)
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        let res: ResponseWrapper<Vec<HashMap<String, String>>> = serde_json::from_str(v.as_str()?)?;
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
        let v = minreq::get(format!(
            "{}/session/{}/element/{}/selected",
            self.url, session_id, element_id
        ))
        .with_header("Content-Type", "application/json")
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        let res: ResponseWrapper<bool> = serde_json::from_str(v.as_str()?)?;
        Ok(res.value)
    }

    pub(crate) fn get_element_attribute(
        &self,
        session_id: &str,
        element_id: &str,
        name: &str,
    ) -> SResult<String> {
        let v = minreq::get(format!(
            "{}/session/{}/element/{}/attribute/{}",
            self.url, session_id, element_id, name
        ))
        .with_header("Content-Type", "application/json")
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        let res: ResponseWrapper<String> = serde_json::from_str(v.as_str()?)?;
        Ok(res.value)
    }

    pub(crate) fn get_element_property(
        &self,
        session_id: &str,
        element_id: &str,
        name: &str,
    ) -> SResult<String> {
        let v = minreq::get(format!(
            "{}/session/{}/element/{}/property/{}",
            self.url, session_id, element_id, name
        ))
        .with_header("Content-Type", "application/json")
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        let res: ResponseWrapper<String> = serde_json::from_str(v.as_str()?)?;
        Ok(res.value)
    }

    pub(crate) fn get_element_css_value(
        &self,
        session_id: &str,
        element_id: &str,
        name: &str,
    ) -> SResult<String> {
        let v = minreq::get(format!(
            "{}/session/{}/element/{}/css/{}",
            self.url, session_id, element_id, name
        ))
        .with_header("Content-Type", "application/json")
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        let res: ResponseWrapper<String> = serde_json::from_str(v.as_str()?)?;
        Ok(res.value)
    }

    pub(crate) fn get_element_text(&self, session_id: &str, element_id: &str) -> SResult<String> {
        let v = minreq::get(format!(
            "{}/session/{}/element/{}/text",
            self.url, session_id, element_id,
        ))
        .with_header("Content-Type", "application/json")
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        let res: ResponseWrapper<String> = serde_json::from_str(v.as_str()?)?;
        Ok(res.value)
    }

    pub(crate) fn get_element_tag_name(
        &self,
        session_id: &str,
        element_id: &str,
    ) -> SResult<String> {
        let v = minreq::get(format!(
            "{}/session/{}/element/{}/name",
            self.url, session_id, element_id,
        ))
        .with_header("Content-Type", "application/json")
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        let res: ResponseWrapper<String> = serde_json::from_str(v.as_str()?)?;
        Ok(res.value)
    }

    pub(crate) fn get_element_rect(&self, session_id: &str, element_id: &str) -> SResult<Rect> {
        let v = minreq::get(format!(
            "{}/session/{}/element/{}/rect",
            self.url, session_id, element_id,
        ))
        .with_header("Content-Type", "application/json")
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        let res: ResponseWrapper<Rect> = serde_json::from_str(v.as_str()?)?;
        Ok(res.value)
    }

    pub(crate) fn is_element_enabled(&self, session_id: &str, element_id: &str) -> SResult<bool> {
        let v = minreq::get(format!(
            "{}/session/{}/element/{}/enabled",
            self.url, session_id, element_id,
        ))
        .with_header("Content-Type", "application/json")
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        let res: ResponseWrapper<bool> = serde_json::from_str(v.as_str()?)?;
        Ok(res.value)
    }

    pub(crate) fn element_click(&self, session_id: &str, element_id: &str) -> SResult<()> {
        let v = minreq::post(format!(
            "{}/session/{}/element/{}/click",
            self.url, session_id, element_id,
        ))
        .with_header("Content-Type", "application/json")
        .with_body("{}")
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        Ok(())
    }

    pub(crate) fn element_clear(&self, session_id: &str, element_id: &str) -> SResult<()> {
        let v = minreq::post(format!(
            "{}/session/{}/element/{}/clear",
            self.url, session_id, element_id,
        ))
        .with_header("Content-Type", "application/json")
        .with_body("{}")
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        Ok(())
    }

    pub(crate) fn element_send_keys(
        &self,
        session_id: &str,
        element_id: &str,
        keys: &str,
    ) -> SResult<()> {

        let v = minreq::post(format!(
            "{}/session/{}/element/{}/value",
            self.url, session_id, element_id,
        ))
        .with_header("Content-Type", "application/json")
        .with_body( format!(
            r#"{{"text":"{keys}","value":[{}]}}"#,
            keys.chars()
                .map(|f| format!(r#""{}""#, f))
                .collect::<Vec<String>>()
                .join(",")
        ))
        .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        Ok(())
    }

    pub(crate) fn get_page_source(&self, session_id: &str) -> SResult<String> {
        let v = minreq::get(format!("{}/session/{}/source", self.url, session_id,))
            .with_header("Content-Type", "application/json")
            .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        let res: ResponseWrapper<String> = serde_json::from_str(v.as_str()?)?;
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

        let v = minreq::post(format!("{}/session/{}/execute/sync", self.url, session_id))
            .with_header("Content-Type", "application/json")
            .with_body(serde_json::to_string(&t)?)
            .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        let res: ResponseWrapper<T> = serde_json::from_str(v.as_str()?)?;
        Ok(res.value)
    }

    pub(crate) fn set_timeouts(&self, session_id: &str, timeout: TimeoutType) -> SResult<()> {
        let mut req = minreq::post(format!("{}/session/{}/timeouts", self.url, session_id))
            .with_header("Content-Type", "application/json");
        // req = match timeout {
        //     TimeoutType::Script(t) => req.with_body(format!(r#"{{"type":"script","ms":{t}}}"#)),
        //     TimeoutType::PageLoad(t) => req.with_body(format!(r#"{{"type":"pageLoad","ms":{t}}}"#)),
        //     TimeoutType::Implicit(t) => req.with_body(format!(r#"{{"type":"implicit","ms":{t}}}"#)),
        // };
        req = match timeout {
            TimeoutType::Script(t) => req.with_body(format!(r#"{{"script":{t}}}"#)),
            TimeoutType::PageLoad(t) => req.with_body(format!(r#"{{"pageLoad":{t}}}"#)),
            TimeoutType::Implicit(t) => req.with_body(format!(r#"{{"implicit":{t}}}"#)),
        };
        let v = req.send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        Ok(())
    }

    pub(crate) fn get_timouts(&self, session_id: &str) -> SResult<Vec<TimeoutType>> {
        let req = minreq::get(format!("{}/session/{}/timeouts", self.url, session_id))
            .with_header("Content-Type", "application/json");

        let v = req.send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }

        let res: ResponseWrapper<HashMap<String, u32>> = serde_json::from_str(v.as_str()?)?;

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

    pub(crate) fn perform_actions<'a>(
        &self,
        session_id: &str,
        req: Vec<ActionRequest<'a>>,
    ) -> SResult<()> {
        let mut map = HashMap::new();
        map.insert("actions", req);
        let j = serde_json::to_string(&map)?;
        println!("json={j}");
        let v = minreq::post(format!("{}/session/{}/actions", self.url, session_id))
            .with_header("Content-Type", "application/json")
            .with_body(j)
            .send()?;

        if v.status_code != 200 {
            return Err(SError::message(v.as_str()?.to_string()));
        }
        println!("actions {}", v.as_str()?);
        Ok(())
    }
}

impl From<minreq::Error> for SError {
    fn from(value: minreq::Error) -> Self {
        SError::message(format!("{}", value))
    }
}

impl From<serde_json::Error> for SError {
    fn from(value: serde_json::Error) -> Self {
        SError::message(format!("{}", value))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::driver::Rect;

    use super::Capability;

    #[test]
    fn test_capability() {
        let r = Rect {
            x: None,
            y: Some(32),
            width: None,
            height: Some(39),
        };
        println!("{}", serde_json::to_string(&r).unwrap());
        let c = Capability {
            browser_name: None,
            platform_name: None,
            always_match: Some(crate::option::FirefoxOption {
                host: None,
                port: None,
                driver: None,
                arguments: vec!["1".to_string(), "2".to_string()],
                exec: None,
                env: HashMap::new(),
            }),
        };

        println!("{c}");
        assert_eq!(
            r#"{"capabilities":{"alwaysMatch":{"browserName": "firefox","moz:firefoxOptions":{"prefs": { "dom.ipc.processCount": 4 },"args":["1","2"] }}}}"#,
            format!("{c}")
        );

        let c: Capability<crate::option::FirefoxOption> = Capability {
            browser_name: None,
            platform_name: None,
            always_match: None,
        };

        println!("{c}");
        assert_eq!(r#"{"capabilities":{"alwaysMatch":{}}}"#, format!("{c}"));
    }
}
