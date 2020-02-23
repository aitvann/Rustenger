use crate::message::{ClientMessage, ServerMessage};
use byteorder::{BigEndian, ByteOrder};
use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

/// Codec for Client -> Server transport
#[derive(Default)]
pub struct ClientWriteCodec;

impl ClientWriteCodec {
    pub fn new() -> Self {
        Self
    }
}

impl Encoder for ClientWriteCodec {
    type Item = ClientMessage;
    type Error = bincode::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let size = bincode::serialized_size(&item)? as usize;
        log::debug!("client encode messae size: {}", size);

        // reaserve for head + body
        dst.reserve(2 + size);
        dst.put_u16(size as u16);

        unsafe {
            let bytes = &mut *(dst.bytes_mut() as *mut [std::mem::MaybeUninit<u8>] as *mut [u8]);
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

/// Codec for Client <- Server transport
#[derive(Default)]
pub struct ClientReadCodec;

impl ClientReadCodec {
    pub fn new() -> Self {
        Self
    }
}

impl Decoder for ClientReadCodec {
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
        log::debug!("client docode message size: {}", size);

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

/// Codec for Server -> Client and Server <- Client transport
#[derive(Default)]
pub struct ServerCodec;

impl ServerCodec {
    pub fn new() -> Self {
        Self
    }
}

impl Encoder for ServerCodec {
    type Item = ServerMessage;
    type Error = bincode::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let size = bincode::serialized_size(&item)? as usize;
        log::debug!("server encode message size: {}", size);

        // reaserve for head + body
        dst.reserve(2 + size);
        dst.put_u16(size as u16);

        unsafe {
            let bytes = &mut *(dst.bytes_mut() as *mut [std::mem::MaybeUninit<u8>] as *mut [u8]);
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
        log::debug!("server decode message size: {}", size);

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
