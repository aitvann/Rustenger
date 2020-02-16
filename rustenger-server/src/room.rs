use crate::client::Client;
use crate::utils::EntryExt;
use chrono::Utc;
use rustenger_shared::{
    account::{Account, Username},
    message::UserMessage,
    RoomName,
};
use std::{collections::HashMap, future::Future, sync::Arc};
use thiserror::Error;
use tokio::sync::{mpsc, Mutex, RwLock};

pub type RoomMsgTx = mpsc::Sender<Client>;
pub type RoomMsgRx = mpsc::Receiver<Client>;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("room '{0}' already exist")]
    RoomAlreadyExist(RoomName),
    #[error("room '{0}' does not exist")]
    RoomDoesNotExits(RoomName),
    #[error("send error: {0}")]
    Send(#[from] mpsc::error::SendError<Client>),
    #[error("bincode error: {0}")]
    Bincode(#[from] bincode::Error),
}

// for rooms it is used RwLock, because it is often used for reading
// - access to ServerRoomMessageTx and rarely for writing - adding a new Room;
// used Mutex for ServerRoomMessage because it is always used for writing
/// A mediator between Rooms, contains links to each room and is accessible from each room
#[derive(Clone)]
pub struct Server {
    links: Arc<RwLock<HashMap<RoomName, Mutex<RoomMsgTx>>>>,
}

impl Server {
    pub fn new() -> Self {
        let raw_links = HashMap::<RoomName, Mutex<RoomMsgTx>>::new();
        let links = Arc::new(RwLock::new(raw_links));
        Self { links }
    }

    /// create link to room with name 'name'
    // pub async fn create_room(self, name: RoomName) -> Result<()> {
    pub fn create_room(self, name: RoomName) -> impl Future<Output = Result<()>> + Send {
        async move {
            log::info!("attempt to create new room '{}'", name);

            let (msg_tx, msg_rx) = mpsc::channel(64);
            let mut lock = self.links.write().await;
            lock.entry(name)
                .vacant()
                .ok_or(Error::RoomAlreadyExist(name))?
                .insert(Mutex::new(msg_tx));

            let room = Room::new(name, msg_rx, self.clone());
            tokio::spawn(room.run());
            Ok(())
        }
    }

    /// remove link to room with name 'name'
    /// 'self' insted of '&self" due to this method used in Drop
    pub async fn revome_room(self, name: RoomName) -> Result<()> {
        log::info!("attempt to remove link to room '{}'", name);

        let mut lock = self.links.write().await;
        lock.entry(name)
            .occupied()
            .ok_or(Error::RoomDoesNotExits(name))?
            .remove();

        Ok(())
    }

    /// inser user 'user' into room with name 'room_name'
    pub async fn insert_user(&self, client: Client, room_name: RoomName) -> Result<()> {
        log::info!(
            "attempt to insert user '{}' to room '{}'",
            client.username(),
            room_name
        );

        let lock = self.links.read().await;
        let msg_tx = lock
            .get(&room_name)
            .ok_or(Error::RoomDoesNotExits(room_name))?;

        let mut msg_tx_lock = msg_tx.lock().await;
        msg_tx_lock.send(client).await.map_err(Error::Send)
    }

    /// build 'Vec' of names of all rooms in the server
    pub async fn rooms(&self) -> Vec<RoomName> {
        self.links.read().await.keys().cloned().collect()
    }
}

pub type Clients = HashMap<Username, Option<Client>>;

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
        log::info!("run room: {}", self.name());

        loop {
            self.accept_client();

            if self.clients.is_empty() {
                // yield the current task to allow to accept
                // another clients if work in single thread
                tokio::task::yield_now().await;
                continue;
            }

            self.update().await;
        }
    }

    /// check if new client is avaliable and accpet it
    fn accept_client(&mut self) {
        use futures::future;

        let recv = future::maybe_done(self.msg_rx.recv());
        futures::pin_mut!(recv);
        if let Some(client) = recv.as_mut().take_output() {
            let client = client.unwrap();
            let username = client.username();

            self.clients.insert(username, Some(client));
            log::info!(
                "accepted client with name '{}' to room '{}'",
                username,
                self.name
            );
        }
    }

    /// selects clients messages and handles them
    async fn update(&mut self) {
        use futures::future::{self, FutureExt};
        use rustenger_shared::message::ClientMessage;

        let iter = self.clients.values_mut().map(|client| {
            let client = client.as_mut().unwrap();
            let adresser = client.account();

            client.read().map(move |m| (adresser, m)).boxed()
        });
        let (adresser, res) = future::select_all(iter).await.0;

        match res {
            Err(e) => log::error!("failed to recieve client message: {}", e),
            Ok(ClientMessage::UserMessage(msg)) => {
                if let Err(e) = self.broadcast(adresser, msg).await {
                    log::error!("failed to broadcast user message: {}", e);
                }
            }
            Ok(ClientMessage::Command(cmd)) => {
                let mut entry = self.clients.entry(adresser.username()).occupied().unwrap();
                let client = entry.get_mut().take().unwrap();

                match client.handle(cmd).await {
                    Err(e) => log::error!("failed to handle command: {}", e),
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

    /// sends messages to all clients except 'account'
    async fn broadcast(&mut self, adresser: Account, text: UserMessage) -> Result<()> {
        use rustenger_shared::message::{AccountMessage, ServerMessage};

        let msg = AccountMessage {
            text,
            adresser,
            utc: Utc::now(),
        };

        for client in self
            .clients
            .values_mut()
            .map(|c| c.as_mut().unwrap())
            .filter(|c| c.username() != adresser.username())
        {
            client.write(ServerMessage::AccountMessage(msg)).await?;
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

        let fut = self.server.clone().revome_room(self.name());
        tokio::spawn(fut);
    }
}
