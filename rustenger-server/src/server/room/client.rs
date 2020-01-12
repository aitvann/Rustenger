use futures::{future::ready, SinkExt, StreamExt};
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
            framed.send(ServerMessage::Response(response));

            return match res {
                Err(_) => continue,
                Ok(acc) => Ok(Some(acc)),
            };
        }

        Ok(None)
    }

    // TODO
    fn log_in(_username: Username, _password: Password) -> Result<Account, SignInError> {
        let err = SignInError::InvalidUserNamePassword;
        Err(err)
    }

    // TODO
    fn sing_up(username: Username, _password: Password) -> Result<Account, SignInError> {
        let color = Color::White;
        let acc = Account { username, color };
        Ok(acc)
    }

    pub async fn update() -> () {}
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client {{ framde: ..., account: {:?} }}", self.account)
    }
}
