use std::fmt::{Debug, Display};

pub enum SError {
    message(String),
}
impl Display for SError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::message(m) => f.write_str(m.as_str()),
            // _ => {f.write_str("unkonwn")}
        }
    }
}
impl Debug for SError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::message(arg0) => f.debug_tuple("message").field(arg0).finish(),
        }
    }
}

impl From<std::io::Error> for SError {
    fn from(value: std::io::Error) -> Self {
        Self::message(value.to_string())
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