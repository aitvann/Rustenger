use rustenger_shared::{codec::ServerCodec, message::{ClientMessage, Command}, Account};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use futures::StreamExt;
use std::fmt;

pub struct Client {
    framed: Framed<TcpStream, ServerCodec>,
    account: Account,
}

impl Client {
    /// log in or sign up user, user can exit at that moment and then Ok(None) is returned
    pub async fn new(stream: TcpStream) -> Result<Option<Self>, bincode::Error> {
        let codec = ServerCodec::new();
        let mut framed = Framed::new(stream, codec);

        if let Some(account) = Self::sign_in(&mut framed).await? {
            let client = Self { framed, account };
            return Ok(Some(client))
        }

        Ok(None)
    }

    async fn sign_in(framed: &mut Framed<TcpStream, ServerCodec>) -> Result<Option<Account>, bincode::Error> {
        while let Some(msg) = framed.next().await.transpose()? {
            use Command::*;

            if let ClientMessage::Command(cmd) = msg {
                match cmd {
                    LogIn(username, password) => continue,
                    SignUp(username, password) => continue,
                    Exit => return Ok(None),
                    _ => continue,
                }
            } 
        }

        Ok(None)
    }

    async fn log_in() -> () {

    }

    async fn sing_up() -> () {

    }

    pub async fn update() -> () {

    }

    // pub fn stream(&mut self) -> &TcpStream {
    //     &mut self.stream
    // }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client {{ framde: ..., account: {:?} }}", self.account)
    }
}
