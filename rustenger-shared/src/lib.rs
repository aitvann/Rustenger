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
    pub fn new(username: Username) -> Self {
        let color = Color::White;
        Self { username, color }
    }

    pub fn with_color(username: Username, color: Color) -> Self {
        Self { username, color }
    }

    pub fn username(&self) -> Username {
        self.username
    }

    pub fn color(&self) -> Color {
        self.color
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
