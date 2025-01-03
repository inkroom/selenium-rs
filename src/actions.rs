use std::{collections::HashMap, rc::Rc};

use bon::Builder;
use serde::{ser::SerializeMap, Serialize, Serializer};

use crate::{
    driver::Session,
    element::Element,
    http::{ActionRequest, Http},
    SResult,
};

pub enum Button {
    LEFT = 0,
    MIDDLE = 1,
    RIGHT = 2,
    BACK = 3,
    FORWARD = 4,
}

impl Serialize for Button {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(match self {
            Button::LEFT => 0,
            Button::MIDDLE => 1,
            Button::RIGHT => 2,
            Button::BACK => 3,
            Button::FORWARD => 4,
        })
    }
}

pub enum ActionType {
    KeyDown,
    KeyUp,
    Pause,
    PointerDown,
    PointerUp,
    PointerMove,
    PointerCancel,
    Scroll,
}
impl ActionType {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            ActionType::KeyDown => "keyDown",
            ActionType::KeyUp => "keyUp",
            ActionType::Pause => "pause",
            ActionType::PointerDown => "pointerDown",
            ActionType::PointerUp => "pointerUp",
            ActionType::PointerMove => "pointerMove",
            ActionType::PointerCancel => "pointerCancel",
            ActionType::Scroll => "scroll",
        }
    }
}

impl Serialize for ActionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

pub enum Origin {
    Viewport,
    Pointer,
    Element(String, String),
}
impl Serialize for Origin {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Origin::Viewport => serializer.serialize_str("viewport"),
            Origin::Pointer => serializer.serialize_str("pointer"),
            Origin::Element(identify, id) => {
                let mut s = serializer.serialize_map(Some(2))?;
                s.serialize_entry(identify.as_str(), id.as_str())?;
                s.serialize_entry("ELEMENT", id.as_str())?;
                s.end()
            }
        }
    }
}

pub(crate) enum Device<'a> {
    Pointer(&'a Pointer),
    Keyboard(&'a Keyboard),
    Wheel(&'a Wheel),
}

/// 鼠标、触摸等操作
#[derive(Serialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct Pointer {
    #[serde(rename(serialize = "type"))]
    pub(crate) _type: ActionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) origin: Option<Origin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) button: Option<Button>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) duration: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) y: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) pressure: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tangential_pressure: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tilt_x: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tilt_y: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) twist: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) altitude_angle: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) azimuth_angle: Option<i32>,
}

impl Default for Pointer {
    fn default() -> Self {
        Self {
            _type: ActionType::KeyDown,
            origin: Default::default(),
            button: Default::default(),
            duration: Default::default(),
            x: Default::default(),
            y: Default::default(),
            width: Default::default(),
            height: Default::default(),
            pressure: Default::default(),
            tangential_pressure: Default::default(),
            tilt_x: Default::default(),
            tilt_y: Default::default(),
            twist: Default::default(),
            altitude_angle: Default::default(),
            azimuth_angle: Default::default(),
        }
    }
}

impl Pointer {
    pub(crate) fn parameters() -> HashMap<String, String> {
        let mut v = HashMap::new();
        v.insert("pointerType".to_string(), "mouse".to_string());
        v
    }

    pub fn press(button: Button) -> Self {
        Pointer::builder()
            .r#type(ActionType::PointerDown)
            .button(button)
            .width(0)
            .height(0)
            .pressure(0)
            .tangential_pressure(0)
            .tilt_x(0)
            .tilt_y(0)
            .twist(0)
            .altitude_angle(0)
            .azimuth_angle(0)
            .build()
    }

    pub fn release(button: Button) -> Self {
        Pointer::builder()
            .r#type(ActionType::PointerUp)
            .button(button)
            .build()
    }

    pub fn move_pointer(element: &Element) -> Self {
        Pointer::builder()
            .r#type(ActionType::PointerMove)
            .origin(Origin::Element(
                element.identify.clone(),
                element.id.clone(),
            ))
            .x(0)
            .y(0)
            .duration(100)
            .width(0)
            .height(0)
            .pressure(0)
            .tangential_pressure(0)
            .tilt_x(0)
            .tilt_y(0)
            .twist(0)
            .altitude_angle(0)
            .azimuth_angle(0)
            .build()
    }

    pub fn pause(duration: u32) -> Self {
        Pointer::builder()
            .r#type(ActionType::Pause)
            .duration(duration)
            .build()
    }
}

#[derive(Serialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct Keyboard {
    #[serde(rename(serialize = "type"))]
    pub(crate) _type: ActionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) duration: Option<u32>,
}

impl Keyboard {
    pub fn key_down(key: &str) -> Self {
        Self {
            _type: ActionType::KeyDown,
            value: Some(key.to_string()),
            duration: None,
        }
    }

    pub fn key_up(key: &str) -> Self {
        Self {
            _type: ActionType::KeyUp,
            value: Some(key.to_string()),
            duration: None,
        }
    }
    pub fn pause(duration: u32) -> Self {
        Self {
            _type: ActionType::Pause,
            value: None,
            duration: Some(duration),
        }
    }
}

///
/// 鼠标滚轮
///
#[derive(Serialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct Wheel {
    #[serde(rename(serialize = "type"))]
    pub(crate) _type: ActionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) y: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) duration: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) delta_x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) delta_y: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) origin: Option<Origin>,
}

pub enum Key {
    Null,
    Cancel, // ^break
    Help,
    BackSpace,
    Tab,
    Clear,
    Return,
    Enter,
    Shift,
    Control,
    Alt,
    Pause,
    Escape,
    Space,
    PageUp,
    PageDown,
    End,
    Home,
    ArrowLeft,
    Left,
    ArrowUp,
    Up,
    ArrowRight,
    Right,
    ArrowDown,
    Down,
    Insert,
    Delete,
    Semicolon,
    Equals,

    NumPad0, // number pad keys
    NumPad1,
    NumPad2,
    NumPad3,
    NumPad4,
    NumPad5,
    NumPad6,
    NumPad7,
    NumPad8,
    NumPad9,
    Multiply,
    Add,
    Separator,
    Subtract,
    Decimal,
    Divide,

    F1, // function keys
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    Command, // Apple command key
    Meta,    // alias for Windows key

    /**
     * Japanese modifier key for switching between full- and half-width
     * characters.
     * @see <https://en.wikipedia.org/wiki/Language_input_keys>
     */
    ZenkakuHankaku,
}

impl Key {
    pub fn as_str(&self) -> &'static str {
        match self {
            Key::Null => "\u{E000}",
            Key::Cancel => "\u{E001}", // ^break
            Key::Help => "\u{E002}",
            Key::BackSpace => "\u{E003}",
            Key::Tab => "\u{E004}",
            Key::Clear => "\u{E005}",
            Key::Return => "\u{E006}",
            Key::Enter => "\u{E007}",
            Key::Shift => "\u{E008}",
            Key::Control => "\u{E009}",
            Key::Alt => "\u{E00A}",
            Key::Pause => "\u{E00B}",
            Key::Escape => "\u{E00C}",
            Key::Space => "\u{E00D}",
            Key::PageUp => "\u{E00E}",
            Key::PageDown => "\u{E00F}",
            Key::End => "\u{E010}",
            Key::Home => "\u{E011}",
            Key::ArrowLeft => "\u{E012}",
            Key::Left => "\u{E012}",
            Key::ArrowUp => "\u{E013}",
            Key::Up => "\u{E013}",
            Key::ArrowRight => "\u{E014}",
            Key::Right => "\u{E014}",
            Key::ArrowDown => "\u{E015}",
            Key::Down => "\u{E015}",
            Key::Insert => "\u{E016}",
            Key::Delete => "\u{E017}",
            Key::Semicolon => "\u{E018}",
            Key::Equals => "\u{E019}",

            Key::NumPad0 => "\u{E01A}", // number pad keys
            Key::NumPad1 => "\u{E01B}",
            Key::NumPad2 => "\u{E01C}",
            Key::NumPad3 => "\u{E01D}",
            Key::NumPad4 => "\u{E01E}",
            Key::NumPad5 => "\u{E01F}",
            Key::NumPad6 => "\u{E020}",
            Key::NumPad7 => "\u{E021}",
            Key::NumPad8 => "\u{E022}",
            Key::NumPad9 => "\u{E023}",
            Key::Multiply => "\u{E024}",
            Key::Add => "\u{E025}",
            Key::Separator => "\u{E026}",
            Key::Subtract => "\u{E027}",
            Key::Decimal => "\u{E028}",
            Key::Divide => "\u{E029}",

            Key::F1 => "\u{E031}", // function keys
            Key::F2 => "\u{E032}",
            Key::F3 => "\u{E033}",
            Key::F4 => "\u{E034}",
            Key::F5 => "\u{E035}",
            Key::F6 => "\u{E036}",
            Key::F7 => "\u{E037}",
            Key::F8 => "\u{E038}",
            Key::F9 => "\u{E039}",
            Key::F10 => "\u{E03A}",
            Key::F11 => "\u{E03B}",
            Key::F12 => "\u{E03C}",

            Key::Command => "\u{E03D}", // Apple command key
            Key::Meta => "\u{E03D}",    // alias for Windows key

            Key::ZenkakuHankaku => "\u{E040}",
        }
    }
}

pub struct Action {
    pub(crate) pointer: Vec<Pointer>,
    pub(crate) keyboard: Vec<Keyboard>,
    pub(crate) wheel: Vec<Wheel>,
    session: Rc<Session>,
    http: Rc<Http>,
}
impl Action {
    pub(crate) fn new(http: Rc<Http>, session: Rc<Session>) -> Self {
        Action {
            pointer: Vec::new(),
            keyboard: Vec::new(),
            wheel: Vec::new(),
            session,
            http,
        }
    }

    pub fn clear(mut self) -> Self {
        self.pointer.clear();
        self
    }

    pub fn press(mut self, button: Button) -> Self {
        self.pointer.push(Pointer::press(button));
        self
    }

    pub fn release(mut self, button: Button) -> Self {
        self.pointer.push(Pointer::release(button));
        self
    }

    pub fn move_pointer(mut self, element: &Element) -> Self {
        self.pointer.push(Pointer::move_pointer(element));
        self
    }

    pub fn click(self, element: Option<&Element>) -> Self {
        if let Some(e) = element {
            self.move_pointer(e)
        } else {
            self
        }
        .press(Button::LEFT)
        .release(Button::LEFT)
    }

    pub fn double_click(self, element: Option<&Element>) -> Self {
        self.click(element)
            .press(Button::LEFT)
            .release(Button::LEFT)
    }
    /// 右键点击
    pub fn context_click(self, element: Option<&Element>) -> Self {
        if let Some(e) = element {
            self.move_pointer(e)
        } else {
            self
        }
        .press(Button::RIGHT)
        .release(Button::RIGHT)
    }

    pub fn add_pointer(mut self, pointer: Pointer) -> Self {
        self.pointer.push(pointer);
        self
    }

    pub fn key_down(mut self, key: &str) -> Self {
        self.keyboard.push(Keyboard::key_down(key));
        self
    }

    pub fn key_up(mut self, key: &str) -> Self {
        self.keyboard.push(Keyboard::key_up(key));
        self
    }

    pub fn key_down_special(mut self, key: Key) -> Self {
        self.keyboard.push(Keyboard::key_down(key.as_str()));
        self
    }

    pub fn key_up_special(mut self, key: Key) -> Self {
        self.keyboard.push(Keyboard::key_up(key.as_str()));
        self
    }

    pub fn key_pause(mut self, duration: u32) -> Self {
        self.keyboard.push(Keyboard::pause(duration));
        self
    }

    pub fn mouse_pause(mut self, duration: u32) -> Self {
        self.pointer.push(Pointer::pause(duration));
        self
    }

    /// 鼠标滚轮
    pub fn scroll(
        mut self,
        x: i32,
        y: i32,
        delta_x: i32,
        delta_y: i32,
        duration: u32,
        origin: Origin,
    ) -> Self {
        self.wheel.push(
            Wheel::builder()
                .r#type(ActionType::Scroll)
                .x(x)
                .y(y)
                .delta_x(delta_x)
                .delta_y(delta_y)
                .duration(duration)
                .origin(origin)
                .build(),
        );
        self
    }

    pub fn perform(&self) -> SResult<()> {
        let mut req = Vec::new();
        if !self.pointer.is_empty() {
            req.push(ActionRequest {
                actions: self.pointer.iter().map(Device::Pointer).collect(),
                parameters: Some(Pointer::parameters()),
                _type: "pointer".to_string(),
                id: "default mouse".to_string(),
            });
        }
        if !self.keyboard.is_empty() {
            req.push(ActionRequest {
                actions: self.keyboard.iter().map(Device::Keyboard).collect(),
                parameters: None,
                _type: "key".to_string(),
                id: "default keyboard".to_string(),
            });
        }
        if !self.wheel.is_empty() {
            req.push(ActionRequest {
                actions: self.wheel.iter().map(Device::Wheel).collect(),
                parameters: None,
                _type: "wheel".to_string(),
                id: "default wheel".to_string(),
            });
        }
        self.http.perform_actions(&self.session.session_id, req)
    }
}
