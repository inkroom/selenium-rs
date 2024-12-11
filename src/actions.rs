use std::{collections::HashMap, rc::Rc};

use bon::{bon, builder, Builder};
use serde::{
    ser::{SerializeMap, SerializeSeq, SerializeStruct},
    Serialize, Serializer,
};

use crate::{
    driver::Session,
    element::{self, Element},
    http::{ActionRequest, Http},
    SResult,
};
// struct ActionOrigin {
//     pub(crate) identify: String,
//     pub(crate) id: String,
// }

// impl Serialize for ActionOrigin {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let mut s = serializer.serialize_map(Some(2))?;

//         s.serialize_entry(self.identify.as_str(), self.id.as_str())?;
//         s.serialize_entry("ELEMENT", self.id.as_str())?;
//         s.end()
//     }
// }

fn serialize_origin<S>(v: &Option<(String, String)>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut s = serializer.serialize_map(Some(2))?;
    let v = v.as_ref().unwrap();
    s.serialize_entry(v.0.as_str(), v.1.as_str())?;
    s.serialize_entry("ELEMENT", v.1.as_str())?;
    s.end()
}

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

pub(crate) enum Device<'a> {
    Pointer(&'a Pointer),
}

/// 鼠标、触摸等操作
#[derive(Serialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct Pointer {
    #[serde(rename(serialize = "type"))]
    pub(crate) _type: ActionType,
    #[serde(
        serialize_with = "serialize_origin",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) origin: Option<(String, String)>,
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
            .origin((element.identify.clone(), element.id.clone()))
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
}

pub struct Action {
    pub(crate) pointer: Vec<Pointer>,
    session: Rc<Session>,
    http: Rc<Http>,
}
impl Action {
    pub(crate) fn new(http: Rc<Http>, session: Rc<Session>) -> Self {
        Action {
            pointer: Vec::new(),
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
                .press(Button::LEFT)
                .release(Button::LEFT)
        } else {
            self.press(Button::LEFT).release(Button::LEFT)
        }
    }

    pub fn add_pointer(mut self, pointer: Pointer) -> Self {
        self.pointer.push(pointer);
        self
    }

    pub fn perform(&self) -> SResult<()> {
        let req = vec![ActionRequest {
            actions: self.pointer.iter().map(|f| Device::Pointer(f)).collect(),
            parameters: Pointer::parameters(),
            _type: "pointer".to_string(),
            id: "default mouse".to_string(),
        }];
        self.http.perform_actions(&self.session.session_id, req)
    }
}
