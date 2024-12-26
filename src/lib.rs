#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
use std::fmt::{Debug, Display};

pub enum SError {
    /// 启动driver错误
    Driver(String),
    /// 普通错误
    Message(String),
    /// http通信错误，比如连接失败，参数格式错误等等
    Http(i32, String),
    /// http请求成功，但是对应参数不正确，比如找不到元素
    Browser(String),
}
impl Display for SError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(m) => f.write_fmt(format_args!("message:{m}")),
            Self::Driver(m) => f.write_fmt(format_args!("driver:{m}")),
            Self::Http(status,m) => f.write_fmt(format_args!("http:status:{status}, reason:{m}")),
            Self::Browser(m) => f.write_fmt(format_args!("browser:{m}")),

        }
    }
}

impl Debug for SError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Driver(arg0) => f.debug_tuple("Driver").field(arg0).finish(),
            Self::Message(arg0) => f.debug_tuple("Message").field(arg0).finish(),
            Self::Http(arg0, arg1) => f.debug_tuple("Http").field(arg0).field(arg1).finish(),
            Self::Browser(arg0) => f.debug_tuple("Browser").field(arg0).finish(),
        }
    }
}

impl From<std::io::Error> for SError {
    fn from(value: std::io::Error) -> Self {
        Self::Message(value.to_string())
    }
}

type SResult<T> = Result<T, SError>;
mod actions;
pub mod driver;
pub mod element;
pub(crate) mod http;
pub mod option;
pub mod shadow;

pub use actions::Key;
pub use actions::Origin;
pub use driver::By;
pub use driver::TimeoutType;
