#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables, unused_mut))]
use std::fmt::{Debug, Display};

// extern crate selenium_manager;
extern {

}

pub enum SError {
    Message(String),
}
impl Display for SError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(m) => f.write_str(m.as_str()),
            // _ => {f.write_str("unkonwn")}
        }
    }
}
impl Debug for SError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(arg0) => f.debug_tuple("message").field(arg0).finish(),
        }
    }
}

impl From<std::io::Error> for SError {
    fn from(value: std::io::Error) -> Self {
        Self::Message(value.to_string())
    }
}

type SResult<T> = Result<T, SError>;
pub mod driver;
pub mod element;
pub(crate) mod http;
pub mod option;
pub mod shadow;
mod actions;

pub use driver::By;
pub use driver::TimeoutType;
pub use actions::Key;
pub use actions::Origin;