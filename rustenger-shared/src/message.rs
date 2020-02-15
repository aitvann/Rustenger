use super::{
    account::{Account, Color, Password, Username},
    RoomName,
};
use arrayvec::ArrayString;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type UserMessage = ArrayString<[u8; 1024]>;

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
    LogIn(Username, Password),
    SignUp(Username, Password),
    CreateRoom(RoomName),
    SelectRoom(RoomName),
    ExitRoom,
    RoomsList,
    SelectColor(Color),
    DeleteAccount,
    Exit,
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
