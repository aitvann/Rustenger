use bytes::BytesMut;
use std::{
    fmt,
    io::{self, BufRead, Read, Write},
};
use tokio_util::codec::{Decoder, Encoder};

mod framed_read;
pub use framed_read::FramedRead;
use framed_read::FramedReadInner;

mod framed_write;
pub use framed_write::FramedWrite;
use framed_write::FramedWriteInner;

pub struct Framed<T, C> {
    inner: FramedReadInner<FramedWriteInner<Fuse<T, C>>>,
}

struct Fuse<T, U> {
    io: T,
    codec: U,
}

impl<T, C> Framed<T, C>
where
    T: Read + Write,
    C: Decoder + Encoder,
{
    /// Provides a `Stream` and `Sink` interface for reading and writing to this
    /// I/O object, using `Decoder` and `Encoder` to read and write the raw data.
    ///
    /// Raw I/O objects work with byte sequences, but higher-level code usually
    /// wants to batch these into meaningful chunks, called "frames". This
    /// method layers framing on top of an I/O object, by using the codec
    /// traits to handle encoding and decoding of messages frames. Note that
    /// the incoming and outgoing frame types may be distinct.
    ///
    /// If you want to work more directly with the streams and sink, consider
    /// calling `split` on the `Framed` returned by this method, which will
    /// break them into separate objects, allowing them to interact more easily.
    pub fn new(io: T, codec: C) -> Self {
        let fuse = Fuse { io, codec };
        let inner_write = FramedWriteInner::new(fuse);
        let inner_read = FramedReadInner::new(inner_write);

        Self { inner: inner_read }
    }

    /// Blocks the current thread until the underlying stream receives enough
    /// bytes to create an item and return it
    pub fn read(&mut self) -> Result<<C as Decoder>::Item, <C as Decoder>::Error> {
        self.inner.read()
    }

    /// Desirealize the item and block the current thread until
    // it writes into the buffer and flushs
    ///
    /// Note that, because of the flushing requirement, it is usually better to batch
    /// together items to send via send_all, rather than flushing between each item.
    pub fn send(&mut self, item: <C as Encoder>::Item) -> Result<(), <C as Encoder>::Error> {
        self.inner.inner.send(item)
    }

    /// drives the iter to keep producing items until it is exhausted,
    /// sending each item and the flushs
    pub fn send_all<I>(&mut self, iter: I) -> Result<(), <C as Encoder>::Error>
    where
        I: Iterator<Item = <C as Encoder>::Item>,
    {
        self.inner.inner.send_all(iter)
    }
}

impl<T, C> Framed<T, C> {
    /// Provides a `Stream` and `Sink` interface for reading and writing to this
    /// I/O object, using `Decoder` and `Encoder` to read and write the raw data.
    ///
    /// Raw I/O objects work with byte sequences, but higher-level code usually
    /// wants to batch these into meaningful chunks, called "frames". This
    /// method layers framing on top of an I/O object, by using the codec
    /// traits to handle encoding and decoding of messages frames. Note that
    /// the incoming and outgoing frame types may be distinct.
    ///
    /// If you want to work more directly with the streams and sink, consider
    /// calling `split` on the `Framed` returned by this method, which will
    /// break them into separate objects, allowing them to interact more easily.
    pub fn from_parts(parts: FramedParts<T, C>) -> Framed<T, C> {
        let fuse = Fuse {
            io: parts.io,
            codec: parts.codec,
        };
        let framed_write = FramedWriteInner::new(fuse);
        let framed_read = FramedReadInner::new(framed_write);

        Self { inner: framed_read }
    }

    /// Returns a reference to the underlying I/O stream wrapped by
    /// `Framed`.
    ///
    /// Note that care should be taken to not tamper with the underlying stream
    /// of data coming in as it may corrupt the stream of frames otherwise
    /// being worked with.
    pub fn get_ref(&self) -> &T {
        &self.inner.get_ref().get_ref().io
    }

    /// Returns a mutable reference to the underlying I/O stream wrapped by
    /// `Framed`.
    ///
    /// Note that care should be taken to not tamper with the underlying stream
    /// of data coming in as it may corrupt the stream of frames otherwise
    /// being worked with.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner.get_mut().get_mut().io
    }

    /// Returns a reference to the underlying codec wrapped by
    /// `Framed`.
    ///
    /// Note that care should be taken to not tamper with the underlying codec
    /// as it may corrupt the stream of frames otherwise being worked with.
    pub fn codec(&self) -> &C {
        &self.inner.get_ref().get_ref().codec
    }

    /// Returns a mutable reference to the underlying codec wrapped by
    /// `Framed`.
    ///
    /// Note that care should be taken to not tamper with the underlying codec
    /// as it may corrupt the stream of frames otherwise being worked with.
    pub fn codec_mut(&mut self) -> &mut C {
        &mut self.inner.get_mut().get_mut().codec
    }

    /// Returns a reference to the read buffer.
    pub fn read_buffer(&self) -> &BytesMut {
        self.inner.buffer()
    }

    /// Consumes the `Framed`, returning its underlying I/O stream.
    ///
    /// Note that care should be taken to not tamper with the underlying stream
    /// of data coming in as it may corrupt the stream of frames otherwise
    /// being worked with.
    pub fn into_inner(self) -> T {
        self.inner.into_inner().into_inner().io
    }

    /// Consumes the `Framed`, returning its underlying I/O stream, the buffer
    /// with unprocessed data, and the codec.
    ///
    /// Note that care should be taken to not tamper with the underlying stream
    /// of data coming in as it may corrupt the stream of frames otherwise
    /// being worked with.
    pub fn into_parts(self) -> FramedParts<T, C> {
        let (inner, read_buf) = self.inner.into_parts();
        let (inner, write_buf) = inner.into_parts();

        FramedParts {
            io: inner.io,
            codec: inner.codec,
            read_buf,
            write_buf,
        }
    }
}

impl<T, U> fmt::Debug for Framed<T, U>
where
    T: fmt::Debug,
    U: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Framed")
            .field("io", &self.inner.get_ref().get_ref().io)
            .field("codec", &self.inner.get_ref().get_ref().codec)
            .finish()
    }
}

// ======== impl Fuse ========

impl<T: Read, U> Read for Fuse<T, U> {
    fn read(&mut self, dst: &mut [u8]) -> io::Result<usize> {
        self.io.read(dst)
    }
}

impl<T: BufRead, U> BufRead for Fuse<T, U> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.io.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.io.consume(amt)
    }
}

impl<T: Write, U> Write for Fuse<T, U> {
    fn write(&mut self, src: &[u8]) -> io::Result<usize> {
        self.io.write(src)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.io.flush()
    }
}

impl<T, U: Decoder> Decoder for Fuse<T, U> {
    type Item = U::Item;
    type Error = U::Error;

    fn decode(&mut self, buffer: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.codec.decode(buffer)
    }

    fn decode_eof(&mut self, buffer: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.codec.decode_eof(buffer)
    }
}

impl<T, U: Encoder> Encoder for Fuse<T, U> {
    type Item = U::Item;
    type Error = U::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.codec.encode(item, dst)
    }
}

/// `FramedParts` contains an export of the data of a Framed transport.
/// It can be used to construct a new `Framed` with a different codec.
/// It contains all current buffers and the inner transport.
#[derive(Debug)]
#[non_exhaustive]
pub struct FramedParts<T, U> {
    /// The inner transport used to read bytes to and write bytes to
    pub io: T,

    /// The codec
    pub codec: U,

    /// The buffer with read but unprocessed data.
    pub read_buf: BytesMut,

    /// A buffer with unprocessed data which are not written yet.
    pub write_buf: BytesMut,
}

impl<T, U> FramedParts<T, U> {
    /// Create a new, default, `FramedParts`.
    pub fn new(io: T, codec: U) -> FramedParts<T, U> {
        let read_buf = BytesMut::new();
        let write_buf = BytesMut::new();

        Self {
            io,
            codec,
            read_buf,
            write_buf,
        }
    }
}
