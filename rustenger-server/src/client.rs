use crate::room::{Error, Result, Server};
use crate::utils::framed_read;
use futures::SinkExt;
use rustenger_shared::{
    account::{Account, Color, Password, Username},
    codec::ServerCodec,
    message::{ClientMessage, Command, Response, ServerMessage, SignInError},
    RoomName,
};
use std::{fmt, result};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub struct Client {
    framed: Framed<TcpStream, ServerCodec>,
    account: Account,
    server: Server,
}

impl Client {
    /// creates new client
    pub async fn new(stream: TcpStream, server: Server) -> Result<Option<Self>> {
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
    async fn sign_in(framed: &mut Framed<TcpStream, ServerCodec>) -> Result<Option<Account>> {
        loop {
            if let ClientMessage::Command(cmd) = framed_read(framed).await? {
                use Command::*;

                let res = match cmd {
                    LogIn(un, pw) => Self::log_in(un, pw),
                    SignUp(un, pw) => Self::sing_up(un, pw),
                    Exit => return Ok(None),
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
    fn log_in(username: Username, _password: Password) -> result::Result<Account, SignInError> {
        log::info!("attempt to log in: {}", username);

        let err = SignInError::InvalidUserNamePassword;
        Err(err)
    }

    // TODO
    /// if an account with the same name does not exist, creates it
    fn sing_up(username: Username, _password: Password) -> result::Result<Account, SignInError> {
        log::info!("attempt to sing up: {}", username);

        let acc = Account::new(username);
        Ok(acc)
    }

    /// reads a message from the user
    pub async fn read(&mut self) -> Result<ClientMessage> {
        framed_read(&mut self.framed).await
    }

    /// sends a message to the user
    pub async fn write(&mut self, msg: ServerMessage) -> Result<()> {
        self.framed.send(msg).await.map_err(Error::Bincode)
    }

    /// returns account
    pub fn account(&self) -> Account {
        self.account
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
    pub async fn run(mut self) -> Result<()> {
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
    pub async fn handle(self, cmd: Command) -> Result<Option<Self>> {
        use Command::*;

        match cmd {
            CreateRoom(rn) => self.create_room(rn).await,
            SelectRoom(rn) => self.select_room(rn).await,
            ExitRoom => self.exit_room().await,
            RoomsList => self.room_list().await,
            SelectColor(c) => self.select_color(c),
            // DeleteAccount => (), TODO
            Exit => self.exit(),
            cmd => {
                log::warn!("unexpected command: {:?}", cmd);
                Ok(Some(self))
            }
        }
    }

    async fn create_room(self, room_name: RoomName) -> Result<Option<Self>> {
        self.server
            .clone()
            .create_room(room_name)
            .await
            .map(|_| Some(self))
    }

    async fn select_room(self, room_name: RoomName) -> Result<Option<Self>> {
        self.server
            .clone()
            .insert_user(self, room_name)
            .await
            .map(|_| None)
    }

    // async fn exit_room(self) -> Result<Option<Self>> {
    fn exit_room(self) -> impl std::future::Future<Output = Result<Option<Self>>> + Send {
        async move {
            tokio::spawn(self.run());
            Ok(None)
        }
    }

    async fn room_list(mut self) -> Result<Option<Self>> {
        let rooms = self.server.rooms().await;
        let response = Response::RoomsList(rooms);
        let serv_message = ServerMessage::Response(response);

        self.write(serv_message).await.map(|_| Some(self))
    }

    fn select_color(mut self, color: Color) -> Result<Option<Self>> {
        self.set_color(color);
        Ok(Some(self))
    }

    fn exit(self) -> Result<Option<Self>> {
        log::info!("user exited");
        Ok(None)
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client {{ framed: .., account: {:?} }}", self.account)
    }
}
