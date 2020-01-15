use futures::{SinkExt, StreamExt};
use rustenger_shared::{
    codec::ServerCodec,
    message::{ClientMessage, Command, Response, ServerMessage, SignInError},
    Account, Color, Password, Username,
};
use std::fmt;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub struct Client {
    framed: Framed<TcpStream, ServerCodec>,
    account: Account,
}

impl Client {
    /// creates new client
    pub async fn new(stream: TcpStream) -> Result<Option<Self>, bincode::Error> {
        let codec = ServerCodec::new();
        let mut framed = Framed::new(stream, codec);

        let client = Self::sign_in(&mut framed)
            .await?
            .map(|account| Self { framed, account });

        Ok(client)
    }

    /// log in or sign up user, user can exit at that moment and then Ok(None) is returned
    async fn sign_in(
        framed: &mut Framed<TcpStream, ServerCodec>,
    ) -> Result<Option<Account>, bincode::Error> {
        use futures::future::ready;

        while let Some(cmd) = framed
            .filter_map(|r| ready(r.map(ClientMessage::command).transpose()))
            .next()
            .await
            .transpose()?
        {
            use Command::*;

            let res = match cmd {
                LogIn(un, pw) => Self::log_in(un, pw),
                SignUp(un, pw) => Self::sing_up(un, pw),
                Exit => return Ok(None),
                _ => continue,
            };

            let response = Response::SignInResult(res.clone().map(|_| ()));
            framed.send(ServerMessage::Response(response)).await?;

            return match res {
                Err(_) => continue,
                Ok(acc) => Ok(Some(acc)),
            };
        }

        Ok(None)
    }

    // TODO
    /// finds an account by name and returns it if the passwords match
    fn log_in(_username: Username, _password: Password) -> Result<Account, SignInError> {
        let err = SignInError::InvalidUserNamePassword;
        Err(err)
    }

    // TODO
    /// if an account with the same name does not exist, creates it
    fn sing_up(username: Username, _password: Password) -> Result<Account, SignInError> {
        let color = Color::White;
        let acc = Account { username, color };
        Ok(acc)
    }

    /// reads a message from the user
    pub async fn read(&mut self) -> Result<(Username, ClientMessage), bincode::Error> {
        self.framed
            .next()
            .await
            .unwrap_or(Ok(ClientMessage::Command(Command::Exit)))
            .map(|r| (self.account.username, r))
    }

    /// sends a message to the user
    pub async fn write(&mut self, msg: ServerMessage) -> Result<(), bincode::Error> {
        self.framed.send(msg).await
    }

    /// returns username
    pub fn username(&self) -> Username {
        self.account.username
    }

    // TODO
    /// runs the client until the user selects a room
    pub async fn run(self) -> Result<Option<Self>, ()> {
        Ok(None)
    }

    // TODO
    /// handles commands
    pub async fn handle(self, cmd: Command) -> Result<Option<Self>, ()> {
        Ok(None)
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client {{ framed: ..., account: {:?} }}", self.account)
    }
}
