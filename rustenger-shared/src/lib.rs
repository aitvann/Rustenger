use arrayvec::ArrayString;
use serde::{Serialize, Deserialize};

pub mod codec;

pub type AccountName = ArrayString<[u8; 64]>; 
pub type RoomName = ArrayString<[u8; 64]>;

#[derive(Serialize, Deserialize)]
pub struct Account {
    name: AccountName,
    color: Color,
}

#[derive(Serialize, Deserialize)]
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
