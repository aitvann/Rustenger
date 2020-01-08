use std::collections::HashMap;
use tokio::sync::mpsc;

mod user;
pub use user::User;

/// message processed by the room and sent by the server
#[derive(Debug, Clone)]
pub enum RoomMessage {
    InsertUser(User),
}

pub type RoomMsgTx = mpsc::Sender<RoomMessage>;
pub type RoomMsgRx = mpsc::Receiver<RoomMessage>;

pub struct Room<'a> {
    pub users: HashMap<&'a str, User>,
    pub msg_rx: RoomMsgRx,
}
