use super::Fuse;
use bytes::BytesMut;
use std::{
    fmt,
    io::{self, Read},
};
use tokio_util::codec::Decoder;

const INITIAL_CAPACITY: usize = 8 * 1024;

pub struct FramedRead<T, U> {
    inner: FramedReadInner<Fuse<T, U>>,
}

pub(super) struct FramedReadInner<T> {
    pub(super) inner: T,
    pub(super) buffer: BytesMut,
}

impl<T, D> FramedRead<T, D>
where
    T: Read,
    D: Decoder,
{
    /// Creates a new `FramedRead` with the given `decoder`.
    pub fn new(inner: T, decoder: D) -> Self {
        let inner = Fuse {
            io: inner,
            codec: decoder,
        };

        let inner = FramedReadInner::new(inner);
        Self { inner }
    }

    /// Blocks the current thread until the underlying stream receives enough
    /// bytes to create an item and return it
    pub fn read(&mut self) -> Result<D::Item, D::Error> {
        self.inner.read()
    }
}

impl<T, D> FramedRead<T, D> {
    /// Returns a reference to the underlying I/O stream wrapped by
    /// `FramedRead`.
    ///
    /// Note that care should be taken to not tamper with the underlying stream
    /// of data coming in as it may corrupt the stream of frames otherwise
    /// being worked with.
    pub fn get_ref(&self) -> &T {
        &self.inner.inner.io
    }

    /// Returns a mutable reference to the underlying I/O stream wrapped by
    /// `FramedRead`.
    ///
    /// Note that care should be taken to not tamper with the underlying stream
    /// of data coming in as it may corrupt the stream of frames otherwise
    /// being worked with.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner.inner.io
    }

    /// Consumes the `FramedRead`, returning its underlying I/O stream.
    ///
    /// Note that care should be taken to not tamper with the underlying stream
    /// of data coming in as it may corrupt the stream of frames otherwise
    /// being worked with.
    pub fn into_inner(self) -> T {
        self.inner.inner.io
    }

    /// Returns a reference to the underlying decoder.
    pub fn decoder(&self) -> &D {
        &self.inner.inner.codec
    }

    /// Returns a mutable reference to the underlying decoder.
    pub fn decoder_mut(&mut self) -> &mut D {
        &mut self.inner.inner.codec
    }

    /// Returns a reference to the read buffer.
    pub fn read_buffer(&self) -> &BytesMut {
        &self.inner.buffer
    }
}

impl<T, D> Iterator for FramedRead<T, D>
where
    T: Read,
    D: Decoder,
{
    type Item = D::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.read().ok()
    }
}

impl<T, D> fmt::Debug for FramedRead<T, D>
where
    T: fmt::Debug,
    D: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FramedRead")
            .field("inner", &self.inner.inner.io)
            .field("decoder", &self.inner.inner.codec)
            .field("buffer", &self.inner.buffer)
            .finish()
    }
}

impl<T> FramedReadInner<T> {
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

    pub(crate) fn buffer(&self) -> &BytesMut {
        &self.buffer
    }

    pub(crate) fn into_parts(self) -> (T, BytesMut) {
        (self.inner, self.buffer)
    }

    pub(super) fn read(&mut self) -> Result<T::Item, T::Error>
    where
        T: Decoder + Read,
    {
        loop {
            // Make sure we've got room for at least one byte to read
            // to ensure that we don't get a spurious 0 that looks like EOF
            self.buffer.reserve(1);

            // fill the buffer until we can read at least one value
            if self.inner.read(&mut self.buffer)? == 0 {
                return self
                    .inner
                    .decode_eof(&mut self.buffer)?
                    .ok_or(io::Error::from(io::ErrorKind::Other).into());
            }

            if let Some(item) = self.inner.decode(&mut self.buffer)? {
                return Ok(item);
            }
        }
    }
}
