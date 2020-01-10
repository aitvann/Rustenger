use super::{Color, Account, RoomName, AccountName};
use tokio_util::codec::{Decoder, Encoder};
use serde::{Serialize, Deserialize};
use bytes::{BytesMut, Buf, BufMut};
use byteorder::{BigEndian, ByteOrder};
use arrayvec::ArrayString;
use chrono::{Utc, DateTime};

pub type MessageText = ArrayString<[u8; 1024]>;

#[derive(Serialize, Deserialize)]
pub struct UserMessage {
    text: MessageText,
    addresser_name: AccountName, 
    utc: DateTime<Utc>, 
}

/// message from client
/// when the client sends Request, it must wait
/// and ignore all messages until it receives a Response
#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    UserMessage(UserMessage),
    Command(Command),
    Request(Request),
}

impl ClientMessage {
    pub fn user_message(&self) -> Option<&UserMessage> {
        match self {
            Self::UserMessage(x) => Some(x),
            _ => None
        }
    }

    pub fn command(&self) -> Option<&Command> {
        match self {
            Self::Command(x) => Some(x),
            _ => None
        }
    }

    pub fn request(&self) -> Option<&Request> {
        match self {
            Self::Request(x) => Some(x),
            _ => None
        }
    }
}

/// command to server, remains without response
#[derive(Serialize, Deserialize)]
pub enum Command {
    RoomsList,
    SelectColor(Color),
}

/// server request
#[derive(Serialize, Deserialize)]
pub enum Request {
    DeleteAccount,
    SelectRoom(RoomName),
}

/// message form server
#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    UserMessage(UserMessage),
    Response(Response),
}

impl ServerMessage {
    pub fn user_message(&self) -> Option<&UserMessage> {
        match self {
            Self::UserMessage(x) => Some(x),
            _ => None
        }
    }

    pub fn response(&self) -> Option<&Response> {
        match self {
            Self::Response(x) => Some(x),
            _ => None
        }
    }
}

/// response to Command::Request
#[derive(Serialize, Deserialize)]
pub enum Response {
    RoomsList(Vec<RoomName>),
    RoomAccountsList(Vec<Account>),
}

/// Codec for Client -> Server transport
pub struct ClientCodec;

impl Encoder for ClientCodec {
    type Item = ClientMessage;
    type Error = bincode::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let size = bincode::serialized_size(&item)? as usize;
        // reaserve for head + body
        dst.reserve(2 + size);
        dst.put_u16(size as u16);

        unsafe {
            let bytes: &mut [u8] = std::mem::transmute(dst.bytes_mut());
            bincode::serialize_into(bytes, &item)?;
            dst.advance_mut(size);
        }

        Ok(())


        // safe but with copy variant:
        // let msg = bincode::serialize(&item)?;
        // let msg_ref: &[u8] = msg.as_ref();

        // dst.reserve(msg_ref.len() + 2);
        // dst.put_u16(msg_ref.len() as u16);
        // dst.put(msg_ref);

        // Ok(())
    }
}

impl Decoder for ClientCodec {
    type Item = ServerMessage;
    type Error = bincode::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // read head
        let size = {
            if src.len() < 2 {
                return Ok(None);
            }
            BigEndian::read_u16(src.as_ref()) as usize
        };

        // reserve bytes for current frame body and next frame head
        src.reserve(size + 2);

        // read body
        if src.len() >= size + 2 {
            src.advance(2);
            let buf = src.split_to(size);
            Ok(Some(bincode::deserialize(&buf)?))
        } else {
            Ok(None)
        }
    }
}

/// Codec for Server -> Client transport
pub struct ServerCodec;

impl Encoder for ServerCodec {
    type Item = ServerMessage;
    type Error = bincode::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let size = bincode::serialized_size(&item)? as usize;
        // reaserve for head + body
        dst.reserve(2 + size);
        dst.put_u16(size as u16);

        unsafe {
            let bytes: &mut [u8] = std::mem::transmute(dst.bytes_mut());
            bincode::serialize_into(bytes, &item)?;
            dst.advance_mut(size);
        }

        Ok(())


        // safe but with copy variant:
        // let msg = bincode::serialize(&item)?;
        // let msg_ref: &[u8] = msg.as_ref();

        // dst.reserve(msg_ref.len() + 2);
        // dst.put_u16(msg_ref.len() as u16);
        // dst.put(msg_ref);

        // Ok(())
    }
}

impl Decoder for ServerCodec {
    type Item = ClientMessage;
    type Error = bincode::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // read head
        let size = {
            if src.len() < 2 {
                return Ok(None);
            }
            BigEndian::read_u16(src.as_ref()) as usize
        };

        // reserve bytes for current frame body and next frame head
        src.reserve(size + 2);

        // read body
        if src.len() >= size + 2 {
            src.advance(2);
            let buf = src.split_to(size);
            Ok(Some(bincode::deserialize(&buf)?))
        } else {
            Ok(None)
        }
    }
}
