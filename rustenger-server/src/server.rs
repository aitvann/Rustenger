use rustenger_shared::RoomName;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};
use thiserror::Error;
use tokio::sync::{mpsc, Mutex, RwLock};

mod room;
use room::{Client, Room, RoomMessage, RoomMsgTx};

#[derive(Error, Debug)]
enum Error {
    #[error("room '{0}' already exist")]
    RoomAlreadyExist(RoomName),
    #[error("room '{0}' does not exist")]
    RoomDoesNotExits(RoomName),
    #[error("send error: {0}")]
    SendError(mpsc::error::SendError<RoomMessage>),
}

// for rooms it is used RwLock, because it is often used for reading
// - access to ServerRoomMessageTx and rarely for writing - adding a new Room;
// used Mutex for ServerRoomMessage because it is always used for writing
/// A mediator between Rooms, contains links to each room and is accessible from each room
#[derive(Clone)]
struct Server {
    links: Arc<RwLock<HashMap<RoomName, Mutex<RoomMsgTx>>>>,
}

impl Server {
    pub fn new() -> Self {
        let raw_links = HashMap::new();
        let links = Arc::new(RwLock::new(raw_links));
        Self { links }
    }

    /// create link to room with name 'name'
    pub async fn create_link(&self, name: &RoomName) -> Result<(), Error> {
        let (msg_tx, msg_rx) = mpsc::channel(64);

        let clients = HashMap::new();
        let _room = Room { clients, msg_rx };

        let mut lock = self.links.write().await;
        match lock.entry(name.clone()) {
            Entry::Occupied(_) => Err(Error::RoomAlreadyExist(name.clone())),
            Entry::Vacant(entry) => {
                entry.insert(Mutex::new(msg_tx));
                // tokio::spawn(room);
                Ok(())
            }
        }
    }

    /// remove link to room with name 'name'
    pub async fn revome_link(&self, name: &RoomName) -> Result<(), Error> {
        let mut lock = self.links.write().await;
        match lock.entry(name.clone()) {
            Entry::Occupied(entry) => {
                entry.remove();
                Ok(())
            }
            Entry::Vacant(_) => Err(Error::RoomDoesNotExits(name.clone())),
        }
    }

    /// inser user 'user' into room with name 'room_name'
    pub async fn insert_user(&self, client: Client, room_name: &RoomName) -> Result<(), Error> {
        let lock = self.links.read().await;
        let msg_tx = lock
            .get(room_name)
            .ok_or(Error::RoomDoesNotExits(room_name.clone()))?;
        let mut msg_tx_lock = msg_tx.lock().await;
        msg_tx_lock
            .send(RoomMessage::InsertClient(client))
            .await
            .map_err(Error::SendError)
    }
}
