use arrayvec::ArrayString;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

pub type Username = ArrayString<[u8; 32]>;
pub type Password = ArrayString<[u8; 32]>;

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

impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let color = match src {
            "Back" => Self::Black,
            "Red" => Self::Red,
            "Green" => Self::Green,
            "Yellow" => Self::Yellow,
            "Blue" => Self::Blue,
            "Magenta" => Self::Magenta,
            "Cyan" => Self::Cyan,
            "White" => Self::White,
            _ => return Err(ParseColorError),
        };

        Ok(color)
    }
}

#[derive(Error, Debug)]
#[error("invalid color name")]
pub struct ParseColorError;
