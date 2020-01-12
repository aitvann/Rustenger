use std::{collections::HashMap, future::Future};
use tokio::sync::mpsc;

mod client;
pub use client::Client;

/// message processed by the room and sent by the server
#[derive(Debug)]
pub enum RoomMessage {
    InsertClient(Client),
}

pub type RoomMsgTx = mpsc::Sender<RoomMessage>;
pub type RoomMsgRx = mpsc::Receiver<RoomMessage>;

pub struct Room<'a> {
    // Not Zero-cost Abstractions
    pub clients: HashMap<&'a str, Box<dyn Future<Output = ()>>>,
    pub msg_rx: RoomMsgRx,
}
