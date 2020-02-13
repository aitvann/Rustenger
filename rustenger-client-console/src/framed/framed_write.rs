use super::Fuse;
use bytes::{Buf, BytesMut};
use std::{
    fmt,
    io::{self, BufRead, Read, Write},
};
use tokio_util::codec::{Decoder, Encoder};

const INITIAL_CAPACITY: usize = 8 * 1024;

pub struct FramedWrite<T, U> {
    inner: FramedWriteInner<Fuse<T, U>>,
}

pub(super) struct FramedWriteInner<T> {
    pub(super) inner: T,
    pub(super) buffer: BytesMut,
}

impl<T, E> FramedWrite<T, E>
where
    T: Write,
    E: Encoder,
{
    /// Creates a new `FramedWrite` with the given `encoder`.
    pub fn new(inner: T, encoder: E) -> Self {
        let inner = Fuse {
            io: inner,
            codec: encoder,
        };

        let inner = FramedWriteInner::new(inner);
        Self { inner }
    }

    /// Desirealize the item and block the current thread until
    // it writes into the buffer and flushs
    ///
    /// Note that, because of the flushing requirement, it is usually better to batch
    /// together items to send via send_all, rather than flushing between each item.
    pub fn send(&mut self, item: E::Item) -> Result<(), E::Error> {
        self.inner.send(item)
    }

    /// drives the iter to keep producing items until it is exhausted,
    /// sending each item and the flushs
    pub fn send_all<I>(&mut self, iter: I) -> Result<(), E::Error>
    where
        I: Iterator<Item = E::Item>,
    {
        self.inner.send_all(iter)
    }
}

impl<T, E> FramedWrite<T, E> {
    /// Returns a reference to the underlying I/O stream wrapped by
    /// `FramedWrite`.
    ///
    /// Note that care should be taken to not tamper with the underlying stream
    /// of data coming in as it may corrupt the stream of frames otherwise
    /// being worked with.
    pub fn get_ref(&self) -> &T {
        &self.inner.inner.io
    }

    /// Returns a mutable reference to the underlying I/O stream wrapped by
    /// `FramedWrite`.
    ///
    /// Note that care should be taken to not tamper with the underlying stream
    /// of data coming in as it may corrupt the stream of frames otherwise
    /// being worked with.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner.inner.io
    }

    /// Consumes the `FramedWrite`, returning its underlying I/O stream.
    ///
    /// Note that care should be taken to not tamper with the underlying stream
    /// of data coming in as it may corrupt the stream of frames otherwise
    /// being worked with.
    pub fn into_inner(self) -> T {
        self.inner.inner.io
    }

    /// Returns a reference to the underlying decoder.
    pub fn encoder(&self) -> &E {
        &self.inner.inner.codec
    }

    /// Returns a mutable reference to the underlying decoder.
    pub fn encoder_mut(&mut self) -> &mut E {
        &mut self.inner.inner.codec
    }
}

impl<T, U> fmt::Debug for FramedWrite<T, U>
where
    T: fmt::Debug,
    U: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FramedWrite")
            .field("inner", &self.inner.get_ref().io)
            .field("encoder", &self.inner.get_ref().codec)
            .field("buffer", &self.inner.buffer)
            .finish()
    }
}

// ======== impl FramedWriteInner ========

impl<T> FramedWriteInner<T> {
    pub(super) fn new(inner: T) -> Self {
        let buffer = BytesMut::with_capacity(INITIAL_CAPACITY);
        Self { inner, buffer }
    }

    pub(super) fn with_capacity(inner: T, capatity: usize) -> Self {
        let buffer = BytesMut::with_capacity(capatity);
        Self { inner, buffer }
    }

    pub(super) fn get_ref(&self) -> &T {
        &self.inner
    }

    pub(super) fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub(super) fn into_inner(self) -> T {
        self.inner
    }

    pub(crate) fn into_parts(self) -> (T, BytesMut) {
        (self.inner, self.buffer)
    }
}

impl<T> FramedWriteInner<T>
where
    T: Encoder + Write,
{
    pub(super) fn send(&mut self, item: T::Item) -> Result<(), T::Error> {
        self.inner.encode(item, &mut self.buffer)?;

        let n = self.inner.write(&self.buffer)?;
        self.buffer.advance(n);

        self.inner.flush()?;
        Ok(())
    }

    pub(super) fn send_all<I>(&mut self, iter: I) -> Result<(), T::Error>
    where
        I: Iterator<Item = T::Item>,
    {
        for item in iter {
            self.inner.encode(item, &mut self.buffer)?;
        }

        let n = self.inner.write(&self.buffer)?;
        self.buffer.advance(n);

        self.inner.flush()?;
        Ok(())
    }
}

impl<T: Read> Read for FramedWriteInner<T> {
    fn read(&mut self, dst: &mut [u8]) -> io::Result<usize> {
        self.inner.read(dst)
    }
}

impl<T: BufRead> BufRead for FramedWriteInner<T> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt)
    }
}

impl<T: Write> Write for FramedWriteInner<T> {
    fn write(&mut self, src: &[u8]) -> io::Result<usize> {
        self.inner.write(src)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Decoder> Decoder for FramedWriteInner<T> {
    type Item = T::Item;
    type Error = T::Error;

    fn decode(&mut self, buffer: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.inner.decode(buffer)
    }

    fn decode_eof(&mut self, buffer: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.inner.decode_eof(buffer)
    }
}

impl<T: Encoder> Encoder for FramedWriteInner<T> {
    type Item = T::Item;
    type Error = T::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.inner.encode(item, dst)
    }
}
