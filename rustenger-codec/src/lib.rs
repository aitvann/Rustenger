use tokio_util::codec::{Decoder, Encoder};
use serde::{Serialize, Deserialize};
use rustenger_shared::{Color, Account, RoomName, AccountName};
use bytes::BytesMut;
use arrayvec::ArrayString;
use chrono::{Utc, DateTime};

pub type MessageText = ArrayString<[u8; 1024]>;

#[derive(Serialize, Deserialize)]
pub struct UserMessage {
    text: MessageText,
    addresser_name: AccountName, 
    utc: DateTime<Utc>, 
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    UserMessage(UserMessage),
    Command(Command),
}

#[derive(Serialize, Deserialize)]
pub enum Command {
    RoomsList,
    SelectColor(Color),
    Request(Request),
}

#[derive(Serialize, Deserialize)]
pub enum Request {
    DeleteAccount,
    SelectRoom(RoomName),
}


#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    UserMessage(UserMessage),
    Response(Response),
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    RoomsList(Vec<RoomName>),
    RoomAccountsList(Vec<Account>),
}

pub struct ClientCodec;

impl Encoder for ClientCodec {
    type Item = ClientMessage;
    type Error = std::io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Decoder for ClientCodec {
    type Item = ServerMessage;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        Ok(None)
    }
}

pub struct ServerCodec;

impl Encoder for ServerCodec {
    type Item = ServerMessage;
    type Error = std::io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Decoder for ServerCodec {
    type Item = ClientMessage;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        Ok(None)
    }
}
