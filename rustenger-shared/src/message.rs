use super::{Color, Account, RoomName, UserName, AccountPassword};
use serde::{Serialize, Deserialize};
use arrayvec::ArrayString;
use chrono::{Utc, DateTime};
use thiserror::Error;

pub type MessageText = ArrayString<[u8; 1024]>;

#[derive(Serialize, Deserialize)]
pub struct UserMessage {
    text: MessageText,
    addresser_name: UserName, 
    utc: DateTime<Utc>, 
}

/// message from client
/// when the client sends Request, it must wait
/// and ignore all messages until it receives a Response
#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    UserMessage(UserMessage),
    Command(Command),
}

impl ClientMessage {
    pub fn user_message(&self) -> Option<&UserMessage> {
        match self {
            Self::UserMessage(x) => Some(x),
            _ => None
        }
    }

    pub fn command(&self) -> Option<&Command> {
        match self {
            Self::Command(x) => Some(x),
            _ => None
        }
    }
}

/// command to server
#[derive(Serialize, Deserialize)]
pub enum Command {
    LogIn(UserName, AccountPassword),
    SignUp(UserName, AccountPassword),
    SelectRoom(RoomName),
    RoomsList,
    SelectColor(Color),
    DeleteAccount,
    LogOut,
    Exit,
}

/// message form server
#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    UserMessage(UserMessage),
    Response(Response),
}

impl ServerMessage {
    pub fn user_message(&self) -> Option<&UserMessage> {
        match self {
            Self::UserMessage(x) => Some(x),
            _ => None
        }
    }

    pub fn response(&self) -> Option<&Response> {
        match self {
            Self::Response(x) => Some(x),
            _ => None
        }
    }
}

/// response to client Request
#[derive(Serialize, Deserialize)]
pub enum Response {
    RoomsList(Vec<RoomName>),
    RoomAccountsList(Vec<Account>),
    LogInResult(Result<(), LogInError>),
    SignUpResult(Result<(), SignUpError>),
}

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum LogInError {
    #[error("invalid username or password")]
    InvalidUserNamePassword,
}

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum SignUpError {
    #[error("this username already used")]
    UserNameAlreadyUsed,
}
