use arrayvec::ArrayString;

pub mod account;
pub mod codec;
pub mod message;

pub type RoomName = ArrayString<[u8; 32]>;
