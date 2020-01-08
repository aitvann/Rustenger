use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};
use thiserror::Error;
use tokio::sync::{mpsc, Mutex, RwLock};

mod room;
use room::{Room, RoomMessage, RoomMsgTx, User};

#[derive(Error, Debug)]
enum ServerError<'a> {
    #[error("room '{0}' already exist")]
    RoomAlreadyExist(&'a str),
    #[error("room '{0}' does not exist")]
    RoomDoesNotExits(&'a str),
    #[error("send error: {0}")]
    SendError(mpsc::error::SendError<RoomMessage>),
}

// for rooms it is used RwLock, because it is often used for reading
// - access to ServerRoomMessageTx and rarely for writing - adding a new Room;
// used Mutex for ServerRoomMessage because it is always used for writing
/// A mediator between Rooms, contains links to each room and is accessible from each room
#[derive(Clone)]
struct Server<'a> {
    links: Arc<RwLock<HashMap<&'a str, Mutex<RoomMsgTx>>>>,
}

impl<'a> Server<'a> {
    pub fn new() -> Self {
        let raw_links = HashMap::new();
        let links = Arc::new(RwLock::new(raw_links));
        Self { links }
    }

    /// create link to room with name 'name'
    pub async fn create_link(&self, name: &'a str) -> Result<(), ServerError<'a>> {
        let (msg_tx, msg_rx) = mpsc::channel(64);

        let users = HashMap::new();
        let _room = Room { users, msg_rx };

        let mut lock = self.links.write().await;
        match lock.entry(name) {
            Entry::Occupied(_) => Err(ServerError::RoomAlreadyExist(name)),
            Entry::Vacant(entry) => {
                entry.insert(Mutex::new(msg_tx));
                // tokio::spawn(room);
                Ok(())
            }
        }
    }

    /// remove link to room with name 'name'
    pub async fn revome_link(&self, name: &'a str) -> Result<(), ServerError<'a>> {
        let mut lock = self.links.write().await;
        match lock.entry(name) {
            Entry::Occupied(entry) => {
                entry.remove();
                Ok(())
            }
            Entry::Vacant(_) => Err(ServerError::RoomDoesNotExits(name)),
        }
    }

    /// inser user 'user' into room with name 'room_name'
    pub async fn insert_user(&self, user: User, room_name: &'a str) -> Result<(), ServerError<'a>> {
        let lock = self.links.read().await;
        let msg_tx = lock
            .get(room_name)
            .ok_or(ServerError::RoomDoesNotExits(room_name))?;
        let mut msg_tx_lock = msg_tx.lock().await;
        msg_tx_lock
            .send(RoomMessage::InsertUser(user))
            .await
            .map_err(ServerError::SendError)
    }
}
