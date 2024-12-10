use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::{
    driver::{Rect, Session},
    http::Http,
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
    pub fn click(&self) -> SResult<()> {
        self.http.element_click(&self.session.session_id, &self.id)
    }
    pub fn clear(&self) -> SResult<()> {
        self.http.element_clear(&self.session.session_id, &self.id)
    }

}

// Interaction

impl Element {}

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
