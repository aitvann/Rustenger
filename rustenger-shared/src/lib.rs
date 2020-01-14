use arrayvec::ArrayString;
use serde::{Deserialize, Serialize};

pub mod codec;
pub mod message;

pub type Username = ArrayString<[u8; 32]>;
pub type Password = ArrayString<[u8; 32]>;
pub type RoomName = ArrayString<[u8; 32]>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub username: Username,
    pub color: Color,
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
