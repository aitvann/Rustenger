use super::{Account, RoomName, Username};
use crate::commands::*;
use arrayvec::ArrayString;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type MessageText = ArrayString<[u8; 1024]>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UserMessage {
    text: MessageText,
    addresser_name: Username,
    utc: DateTime<Utc>,
}

/// message from client
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ClientMessage {
    UserMessage(UserMessage),
    Command(Command),
}

impl ClientMessage {
    pub fn user_message(self) -> Option<UserMessage> {
        match self {
            Self::UserMessage(x) => Some(x),
            _ => None,
        }
    }

    pub fn command(self) -> Option<Command> {
        match self {
            Self::Command(x) => Some(x),
            _ => None,
        }
    }
}

/// command to server
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Command {
    LogIn(LogIn),
    SignUp(SignUp),
    SelectRoom(SelectRoom),
    RoomsList(RoomsList),
    SelectColor(SelectColor),
    DeleteAccount(DeleteAccount),
    LogOut(LogOut),
    Exit(Exit),
}

/// message form server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    UserMessage(UserMessage),
    Response(Response),
}

impl ServerMessage {
    pub fn user_message(self) -> Option<UserMessage> {
        match self {
            Self::UserMessage(x) => Some(x),
            _ => None,
        }
    }

    pub fn response(self) -> Option<Response> {
        match self {
            Self::Response(x) => Some(x),
            _ => None,
        }
    }
}

/// response to client Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    RoomsList(Vec<RoomName>),
    RoomAccountsList(Vec<Account>),
    SignInResult(Result<(), SignInError>),
}

#[derive(Error, Clone, Debug, Serialize, Deserialize)]
pub enum SignInError {
    #[error("invalid username or password")]
    InvalidUserNamePassword,
    #[error("this username already used")]
    UserNameAlreadyUsed,
}
