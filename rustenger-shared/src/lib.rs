use arrayvec::ArrayString;
use serde::{Serialize, Deserialize};

pub mod codec;
pub mod message;

pub type UserName = ArrayString<[u8; 32]>; 
pub type AccountPassword = ArrayString<[u8; 32]>;
pub type RoomName = ArrayString<[u8; 32]>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    name: UserName,
    color: Color,
}

#[derive(Debug, Serialize, Deserialize)]
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
