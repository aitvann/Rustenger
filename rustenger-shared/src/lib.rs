use arrayvec::ArrayString;
use serde::{Deserialize, Serialize};

pub mod codec;
pub mod commands;
pub mod message;

pub type Username = ArrayString<[u8; 32]>;
pub type Password = ArrayString<[u8; 32]>;
pub type RoomName = ArrayString<[u8; 32]>;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Account {
    username: Username,
    color: Color,
}

impl Account {
    /// create new 'Account' with white color
    pub fn new(username: Username) -> Self {
        let color = Color::White;
        Self { username, color }
    }

    /// create new 'Account' with color
    pub fn with_color(username: Username, color: Color) -> Self {
        Self { username, color }
    }

    /// return username
    pub fn username(&self) -> Username {
        self.username
    }

    /// return current color
    pub fn color(&self) -> Color {
        self.color
    }

    /// sets new color
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}
