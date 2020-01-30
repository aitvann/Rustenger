use crate::client::Client;
use crate::utils::EntryExt;
use rustenger_shared::{message::UserMessage, RoomName, Username};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use tokio::sync::{mpsc, Mutex, RwLock};

pub type RoomMsgTx = mpsc::Sender<Client>;
pub type RoomMsgRx = mpsc::Receiver<Client>;

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
        let raw_links = HashMap::<RoomName, Mutex<RoomMsgTx>>::new();
        let links = Arc::new(RwLock::new(raw_links));
        Self { links }
    }

    /// create link to room with name 'name'
    pub async fn create_room(&self, name: RoomName) -> Result<(), Error> {
        log::info!("create new room: {}", name);

        let (msg_tx, msg_rx) = mpsc::channel(64);
        let room = Room::new(name, msg_rx, self.clone());

        let mut lock = self.links.write().await;
        lock.entry(name)
            .vacant()
            .ok_or(Error::RoomAlreadyExist(name))?
            .insert(Mutex::new(msg_tx));

        tokio::spawn(room.run());
        Ok(())
    }

    /// remove link to room with name 'name'
    pub async fn revome_link(self, name: RoomName) -> Result<(), Error> {
        log::info!("remove link to room: {}", name);

        let mut lock = self.links.write().await;
        lock.entry(name)
            .occupied()
            .ok_or(Error::RoomDoesNotExits(name))?
            .remove();

        Ok(())
    }

    /// inser user 'user' into room with name 'room_name'
    pub async fn insert_user(&self, client: Client, room_name: RoomName) -> Result<(), Error> {
        log::info!("insert user {} to room {}", client.username(), room_name);

        let lock = self.links.read().await;
        let msg_tx = lock
            .get(&room_name)
            .ok_or(Error::RoomDoesNotExits(room_name))?;

        let mut msg_tx_lock = msg_tx.lock().await;
        msg_tx_lock.send(client).await.map_err(Error::Send)
    }
}

#[derive(Error, Debug)]
enum Error {
    #[error("room '{0}' already exist")]
    RoomAlreadyExist(RoomName),
    #[error("room '{0}' does not exist")]
    RoomDoesNotExits(RoomName),
    #[error("send error: {0}")]
    Send(#[from] mpsc::error::SendError<Client>),
}

pub type Clients = HashMap<RoomName, Option<Client>>;

pub struct Room {
    name: RoomName,
    clients: Clients,
    msg_rx: RoomMsgRx,
    server: Server,
}

impl Room {
    /// creates new room without links with other rooms
    fn new(name: RoomName, msg_rx: RoomMsgRx, server: Server) -> Self {
        let clients = HashMap::new();
        Self {
            name,
            clients,
            msg_rx,
            server,
        }
    }

    /// runs the room
    pub async fn run(mut self) {
        use futures::future::{self, FutureExt};
        use rustenger_shared::message::ClientMessage;

        log::info!("run room: {}", self.name());

        loop {
            {
                let recv = future::maybe_done(self.msg_rx.recv());
                futures::pin_mut!(recv);
                if let Some(client) = recv.as_mut().take_output() {
                    let client = client.unwrap();
                    self.clients.insert(client.username(), Some(client));
                }
            }

            let iter = self.clients.iter_mut().map(|(name, client)| {
                client
                    .as_mut()
                    .unwrap()
                    .read()
                    .map(move |m| (*name, m))
                    .boxed()
            });
            let (name, res) = future::select_all(iter).await.0;

            match res {
                Err(e) => {
                    log::error!("failed to recieve client message: {}", e);
                    continue;
                }
                Ok(ClientMessage::UserMessage(msg)) => {
                    if let Err(e) = self.broadcast(msg, name).await {
                        log::error!("failed to broadcast user message: {}", e);
                        continue;
                    }
                }
                Ok(ClientMessage::Command(cmd)) => {
                    let mut entry = self.clients.entry(name).occupied().unwrap();
                    let client = entry.get_mut().take().unwrap();

                    match client.handle(cmd).await {
                        Err(e) => {
                            log::error!("failed to handle command: {}", e);
                            continue;
                        }
                        Ok(None) => {
                            entry.remove();
                        }
                        Ok(Some(client)) => {
                            *entry.get_mut() = Some(client);
                        }
                    }
                }
            }
        }
    }

    /// sends messages to all clients except 'skip'
    async fn broadcast(&mut self, msg: UserMessage, skip: Username) -> Result<(), bincode::Error> {
        use rustenger_shared::message::ServerMessage;

        for client in self
            .clients
            .values_mut()
            .map(|c| c.as_mut().unwrap())
            .filter(|c| c.username() != skip)
        {
            client.write(ServerMessage::UserMessage(msg)).await?;
        }

        Ok(())
    }

    pub fn name(&self) -> RoomName {
        self.name
    }
}

impl Drop for Room {
    fn drop(&mut self) {
        log::debug!("drop the room: {}", self.name());

        let fut = self.server.clone().revome_link(self.name());
        tokio::spawn(fut);
    }
}
