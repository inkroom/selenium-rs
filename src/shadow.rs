use crate::{driver::Session, element::Element, http::Http, By, SResult};
use std::{
    fmt::{Debug, Display},
    rc::Rc,
};
pub struct Shadow {
    pub(crate) http: Rc<Http>,
    pub(crate) session: Rc<Session>,
    pub(crate) identify: String,
    pub(crate) id: String,
}
impl Debug for Shadow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Element")
            .field("identify", &self.identify)
            .field("id", &self.id)
            .finish()
    }
}
impl Display for Shadow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("identify=")?;
        f.write_str(&self.identify)?;
        f.write_str(",id=")?;
        f.write_str(&self.id)
        // f.write_str(format!(""))
    }
}

impl Shadow {
    pub fn find_element(&self, css: &str) -> SResult<Element> {
        // TODO 2024-12-10 除了css之外的查找方式都报错了，似乎都有问题，只能暂时取消
        let v = self.http.find_element_from_shadow_root(
            &self.session.session_id,
            &self.id,
            By::Css(css),
        )?;
        Ok(Element {
            http: Rc::clone(&self.http),
            session: Rc::clone(&self.session),
            identify: v.0,
            id: v.1,
        })
    }

    pub fn find_elements(&self, css: &str) -> SResult<Vec<Element>> {
        // TODO 2024-12-10 除了css之外的查找方式都报错了，似乎都有问题，只能暂时取消
        let v = self.http.find_elements_from_shadow_root(
            &self.session.session_id,
            &self.id,
            By::Css(css),
        )?;
        Ok(v.iter()
            .map(|f| Element {
                http: Rc::clone(&self.http),
                session: Rc::clone(&self.session),
                identify: f.0.clone(),
                id: f.1.clone(),
            })
            .collect())
    }
}
