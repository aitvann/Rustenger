use rustenger_shared::{message::UserMessage, RoomName, Username};
use std::collections::HashMap;
use tokio::sync::mpsc;

mod client;
pub use client::Client;

pub type RoomMsgTx = mpsc::Sender<Client>;
pub type RoomMsgRx = mpsc::Receiver<Client>;

pub type Clients = HashMap<RoomName, Client>;

pub struct Room {
    pub clients: Clients,
    pub msg_rx: RoomMsgRx,
}

impl Room {
    /// creates new room without links with other rooms
    pub(super) fn new(msg_rx: RoomMsgRx) -> Self {
        let clients = HashMap::new();
        Self { clients, msg_rx }
    }

    /// runs the room
    pub async fn run(mut self) {
        use futures::{
            future::{self, FutureExt},
            pin_mut,
        };
        use rustenger_shared::message::ClientMessage;

        loop {
            // let iter = self.clients.values_mut().map(|c| c.read().boxed());
            // let (res, _, _) = futures::select! {
            //     client = self.msg_rx.recv().fuse() => {
            //         let client = client.unwrap();
            //         self.clients.insert(client.name(), client);
            //         continue
            //     },
            //     res = future::select_all(iter).fuse() => res,
            // };
            {
                let recv = future::maybe_done(self.msg_rx.recv());
                pin_mut!(recv);
                if let Some(client) = recv.as_mut().take_output() {
                    let client = client.unwrap();
                    self.clients.insert(client.username(), client);
                }
            }

            let iter = self.clients.values_mut().map(|c| c.read().boxed());
            let res = future::select_all(iter).await.0;

            match res {
                Err(_e) => continue,
                Ok((name, ClientMessage::UserMessage(msg))) => {
                    if let Err(_e) = self.broadcast(msg, name).await {
                        continue;
                    }
                }
                Ok((name, ClientMessage::Command(cmd))) => {
                    let client = self.clients.remove(&name).unwrap();
                    match client.handle(cmd).await {
                        Err(_e) => continue,
                        Ok(None) => continue,
                        Ok(Some(client)) => {
                            self.clients.insert(name, client);
                        }
                    }
                }
            }
        }
    }

    /// sends messages to all clients except 'skip'
    async fn broadcast(&mut self, msg: UserMessage, skip: Username) -> Result<(), bincode::Error> {
        use rustenger_shared::message::ServerMessage;

        for client in self.clients.values_mut().filter(|c| c.username() != skip) {
            client.write(ServerMessage::UserMessage(msg)).await?;
        }

        Ok(())
    }
}
