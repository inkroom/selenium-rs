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
            Self::Http(status, m) => f.write_fmt(format_args!("http:status:{status}, reason:{m}")),
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

pub mod base64 {
    use std::{collections::HashMap, sync::OnceLock};

    const B64: [char; 65] = [
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
        'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1',
        '2', '3', '4', '5', '6', '7', '8', '9', '+', '/', '=',
    ];

    //base64查表
    fn base64_map() -> &'static HashMap<u8, u8> {
        static HASHMAP: OnceLock<HashMap<u8, u8>> = OnceLock::new();
        HASHMAP.get_or_init(|| {
            let mut m = HashMap::new();
            for i in 0..65 {
                m.insert(B64[i] as u8, i as u8);
            }
            m
        })
    }

    pub fn encode(input: &[u8]) -> String {
        const BASE64_CHARS: &[u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = Vec::new();
        let mut i = 0;
        let len = input.len();

        while i < len {
            let byte1 = input[i];
            let byte2 = if i + 1 < len { input[i + 1] } else { 0 };
            let byte3 = if i + 2 < len { input[i + 2] } else { 0 };

            let index1 = (byte1 >> 2) as usize;
            let index2 = (((byte1 & 0x03) << 4) | (byte2 >> 4)) as usize;
            let index3 = (((byte2 & 0x0f) << 2) | (byte3 >> 6)) as usize;
            let index4 = (byte3 & 0x3f) as usize;

            result.push(BASE64_CHARS[index1]);
            result.push(BASE64_CHARS[index2]);

            if i + 1 < len {
                result.push(BASE64_CHARS[index3]);
            } else {
                result.push(b'=');
            }

            if i + 2 < len {
                result.push(BASE64_CHARS[index4]);
            } else {
                result.push(b'=');
            }

            i += 3;
        }

        String::from_utf8(result).unwrap()
    }

    pub fn decode(data: &[u8]) -> Vec<u8> {
        let lens = data.len();
        let mut data = data.to_vec();
        for i in 0..lens {
            data[i] = base64_map()[&data[i]];
        }
        let mut sub_count = 0;
        let mut i = lens.saturating_sub(1); // 从末尾开始检查，确保不越界
                                            // 逐个检查字节
        while data[i] == 64 {
            data[i] = 0; // 设置为0
            sub_count += 1; // 计数加1
            i -= 1; // 向前移动
        }

        //向量的分配可以一开始就确定容量
        let capacity = lens * 3 / 4;
        let mut result = Vec::with_capacity(capacity);
        let lens = lens / 4;

        //按位操作，还原字节
        for index in 0..lens {
            let a1 = data[index * 4] << 2 | data[index * 4 + 1] >> 4;
            let a2 = data[index * 4 + 1] << 4 | data[index * 4 + 2] >> 2;
            let a3 = data[index * 4 + 2] << 6 | data[index * 4 + 3];
            result.push(a1);
            result.push(a2);
            result.push(a3);
        }

        //去掉填充的字符
        for _i in 0..sub_count {
            result.pop();
        }
        result
    }
}
