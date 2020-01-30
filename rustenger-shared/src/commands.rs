use crate::{Color, Password, RoomName, Username};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LogIn(pub Username, pub Password);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SignUp(pub Username, pub Password);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SelectRoom(pub RoomName);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RoomsList;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SelectColor(pub Color);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DeleteAccount;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LogOut;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Exit;
