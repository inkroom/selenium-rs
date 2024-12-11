use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::{
    driver::{Rect, Session},
    http::{ActionRequest, Http},
    shadow::Shadow,
    By, SResult,
};

pub enum SendKey {
    Text(String),
}

pub struct Element {
    pub(crate) http: Rc<Http>,
    pub(crate) session: Rc<Session>,
    pub(crate) identify: String,
    pub(crate) id: String,
}
impl Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Element")
            .field("identify", &self.identify)
            .field("id", &self.id)
            .finish()
    }
}
impl Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("identify=")?;
        f.write_str(&self.identify)?;
        f.write_str(",id=")?;
        f.write_str(&self.id)
        // f.write_str(format!(""))
    }
}

impl Element {
    pub fn find_element<'a>(&self, by: By<'a>) -> SResult<Element> {
        let v = self
            .http
            .find_element_from_element(&self.session.session_id, &self.id, by)?;
        Ok(Element {
            http: Rc::clone(&self.http),
            session: Rc::clone(&self.session),
            identify: v.0,
            id: v.1,
        })
    }

    pub fn find_elements<'a>(&self, by: By<'a>) -> SResult<Vec<Element>> {
        let v = self
            .http
            .find_elements_from_element(&self.session.session_id, &self.id, by)?;
        Ok(v.iter()
            .map(|f| Element {
                http: Rc::clone(&self.http),
                session: Rc::clone(&self.session),
                identify: f.0.clone(),
                id: f.1.clone(),
            })
            .collect())
    }

    pub fn get_shadow_root(&self) -> SResult<Shadow> {
        let v = self
            .http
            .get_element_shadow_root(&self.session.session_id, &self.id)?;
        Ok(Shadow {
            http: Rc::clone(&self.http),
            session: Rc::clone(&self.session),
            identify: v.0,
            id: v.1,
        })
    }

    pub fn is_selected(&self) -> SResult<bool> {
        self.http
            .is_element_selected(&self.session.session_id, &self.id)
    }

    pub fn get_attribute(&self, name: &str) -> SResult<String> {
        self.http
            .get_element_attribute(&self.session.session_id, &self.id, name)
    }
    pub fn get_property(&self, name: &str) -> SResult<String> {
        self.http
            .get_element_property(&self.session.session_id, &self.id, name)
    }
    pub fn get_css_value(&self, name: &str) -> SResult<String> {
        self.http
            .get_element_css_value(&self.session.session_id, &self.id, name)
    }
    pub fn get_text(&self) -> SResult<String> {
        self.http
            .get_element_text(&self.session.session_id, &self.id)
    }

    pub fn get_tag_name(&self) -> SResult<String> {
        self.http
            .get_element_tag_name(&self.session.session_id, &self.id)
    }
    pub fn get_rect(&self) -> SResult<Rect> {
        self.http
            .get_element_rect(&self.session.session_id, &self.id)
    }
    pub fn is_enabled(&self) -> SResult<bool> {
        self.http
            .is_element_enabled(&self.session.session_id, &self.id)
    }
    /// 左键点击元素
    pub fn click(&self) -> SResult<()> {
        self.http.element_click(&self.session.session_id, &self.id)
    }
    pub fn clear(&self) -> SResult<()> {
        self.http.element_clear(&self.session.session_id, &self.id)
    }
    /// 发送key，可以当做键盘输入
    pub fn send_keys(&self,key:&str)->SResult<()>{
        self.http.element_send_keys(&self.session.session_id,&self.id,key)
    }
}

// Interaction

// impl Element {
//     // 左键点击元素
//     pub fn click(&self) -> SResult<()> {
//         let actions = vec![
//             InternalAction::builder()
//                 .origin((self.identify.clone(), self.id.clone()))
//                 .r#type("pointerMove".to_string())
//                 .duration(100)
//                 .x(0)
//                 .y(0)
//                 .width(0)
//                 .height(0)
//                 .pressure(0)
//                 .tangential_pressure(0)
//                 .tilt_x(0)
//                 .tilt_y(0)
//                 .twist(0)
//                 .altitude_angle(0)
//                 .azimuth_angle(0)
//                 .build(),
//             InternalAction::builder()
//                 .r#type("pointerDown".to_string())
//                 .button(0)
//                 .width(0)
//                 .height(0)
//                 .pressure(0)
//                 .tangential_pressure(0)
//                 .tilt_x(0)
//                 .tilt_y(0)
//                 .twist(0)
//                 .altitude_angle(0)
//                 .azimuth_angle(0)
//                 .build(),
//             InternalAction::builder()
//                 .r#type("pointerUp".to_string())
//                 .button(0)
//                 .build(),
//         ];

//         let par = HashMap::new();
//         par.insert("pointerType".to_string(), "mouse".to_string());

//         let req = ActionRequest {
//             parameters: par,
//             actions,
//             _type: "pointer".to_string(),
//             id: "default mouse".to_string(),
//         };

//         self.http.perform_actions(&self.session.session_id, req)
//     }
// }

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use crate::{driver::Driver, option::FirefoxBuilder, By, SError};

    fn new_driver() -> Driver {
        let option = FirefoxBuilder::new().host("127.0.0.1").port(3824).build();

        let d = Driver::new(option).unwrap();
        d.get(
            format!(
                "file://{}/demo.html",
                std::env::current_dir()
                    .map_err(|f| SError::message(f.to_string()))
                    .unwrap()
                    .display()
            )
            .as_str(),
        )
        .unwrap();
        sleep(Duration::from_secs(5));
        d
    }

    // #[test]
    fn test_get_attribute() {
        let driver = new_driver();

        let ele = driver.find_element(By::Css("#checkbox")).unwrap();

        assert_eq!("1", ele.get_attribute("value").unwrap());

        driver.quit().unwrap();
    }
}
