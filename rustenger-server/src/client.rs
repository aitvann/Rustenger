use crate::room::{self, Server};
use crate::utils::framed_read;
use async_trait::async_trait;
use futures::SinkExt;
use rustenger_shared::{
    codec::ServerCodec,
    commands,
    message::{ClientMessage, Command, Response, ServerMessage, SignInError},
    Color, Account, Password, Username,
};
use std::fmt;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub struct Client {
    framed: Framed<TcpStream, ServerCodec>,
    account: Account,
    server: Server,
}

impl Client {
    /// creates new client
    pub async fn new(stream: TcpStream, server: Server) -> Result<Option<Self>, bincode::Error> {
        let codec = ServerCodec::new();
        let mut framed = Framed::new(stream, codec);

        let client = Self::sign_in(&mut framed).await?.map(|account| Self {
            framed,
            account,
            server,
        });

        Ok(client)
    }

    /// log in or sign up user, user can exit at that moment and then Ok(None) is returned
    async fn sign_in(
        framed: &mut Framed<TcpStream, ServerCodec>,
    ) -> Result<Option<Account>, bincode::Error> {
        loop {
            if let ClientMessage::Command(cmd) = framed_read(framed).await? {
                use Command::*;

                let res = match cmd {
                    LogIn(commands::LogIn(un, pw)) => Self::log_in(un, pw),
                    SignUp(commands::SignUp(un, pw)) => Self::sing_up(un, pw),
                    Exit(_) => return Ok(None),
                    cmd => {
                        log::warn!("untreated command: {:?}", cmd);
                        continue;
                    }
                };

                let response = Response::SignInResult(res.clone().map(|_| ()));
                framed.send(ServerMessage::Response(response)).await?;

                return match res {
                    Err(e) => {
                        log::warn!("faieled to sign in user: {}", e);
                        continue;
                    }
                    Ok(acc) => Ok(Some(acc)),
                };
            }
        }
    }

    // TODO
    /// finds an account by name and returns it if the passwords match
    fn log_in(username: Username, _password: Password) -> Result<Account, SignInError> {
        log::info!("attempt to log in: {}", username);

        let err = SignInError::InvalidUserNamePassword;
        Err(err)
    }

    // TODO
    /// if an account with the same name does not exist, creates it
    fn sing_up(username: Username, _password: Password) -> Result<Account, SignInError> {
        log::info!("attempt to sing up: {}", username);

        let acc = Account::new(username);
        Ok(acc)
    }

    /// reads a message from the user
    pub async fn read(&mut self) -> Result<ClientMessage, bincode::Error> {
        framed_read(&mut self.framed).await
    }

    /// sends a message to the user
    pub async fn write(&mut self, msg: ServerMessage) -> Result<(), bincode::Error> {
        self.framed.send(msg).await
    }

    /// returns username
    pub fn username(&self) -> Username {
        self.account.username()
    }

    /// sets new color for its account
    pub fn set_color(&mut self, color: Color) {
        self.account.set_color(color)
    }

    /// runs the client until the user selects a room
    pub async fn run(mut self) -> Result<(), Error> {
        log::info!("run client: {}", self.username());

        loop {
            if let ClientMessage::Command(cmd) = self.read().await? {
                match self.handle(cmd).await? {
                    None => return Ok(()),
                    Some(client) => {
                        self = client;
                    }
                }
            }
        }
    }

    /// handle commands
    pub async fn handle(self, cmd: Command) -> Result<Option<Self>, Error> {
        use Command::*;

        match cmd {
            // LogIn(x) => x.handle(self),
            // SignUp(x) => x.handle(self),
            SelectRoom(x) => x.handle(self).await,
            RoomsList(x) => x.handle(self).await,
            SelectColor(x) => x.handle(self).await,
            DeleteAccount(x) => x.handle(self).await,
            Exit(x) => x.handle(self).await,
            cmd => {
                log::warn!("unexpected command: {:?}", cmd);
                Ok(Some(self))
            }
        }
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client {{ framed: .., account: {:?} }}", self.account)
    }
}

/// trait for processing client commands
#[async_trait]
trait Handle {
    async fn handle(self, client: Client) -> Result<Option<Client>, Error>;
}

#[async_trait]
impl Handle for commands::SelectRoom {
    async fn handle(self, client: Client) -> Result<Option<Client>, Error> {
        client
            .server
            .clone()
            .insert_user(client, self.0)
            .await
            .map_err(Error::Server)
            .map(|_| None)
    }
}

#[async_trait]
impl Handle for commands::RoomsList {
    async fn handle(self, mut client: Client) -> Result<Option<Client>, Error> {
        let rooms = client.server.rooms().await;
        let response = Response::RoomsList(rooms);
        let serv_message = ServerMessage::Response(response);

        client
            .write(serv_message)
            .await
            .map_err(Error::Bincode)
            .map(|_| Some(client))
    }
}

#[async_trait]
impl Handle for commands::SelectColor {
    async fn handle(self, mut client: Client) -> Result<Option<Client>, Error> {
        client.set_color(self.0);
        Ok(Some(client))
    }
}

#[async_trait]
// TODO
impl Handle for commands::DeleteAccount {
    async fn handle(self, client: Client) -> Result<Option<Client>, Error> {
        Ok(Some(client))
    }
}

#[async_trait]
impl Handle for commands::Exit {
    async fn handle(self, _client: Client) -> Result<Option<Client>, Error> {
        log::info!("user exited");
        Ok(None)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("bincode error: {0}")]
    Bincode(#[from] bincode::Error),
    #[error("server error: {0}")]
    Server(#[from] room::Error),
}
