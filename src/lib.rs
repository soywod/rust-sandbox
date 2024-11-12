pub mod starttls;
pub mod stream;

use std::{
    io::Result,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Stream<S>(S);

impl<S> Stream<S> {
    pub fn new(stream: S) -> Self {
        Self(stream)
    }
}

pub trait StreamExt<S, const ASYNC: bool> {
    type Context<'a>;
    type Return<T>;

    fn read(&mut self, cx: &mut Self::Context<'_>, buf: &mut [u8]) -> Self::Return<Result<usize>>;

    fn write(&mut self, cx: &mut Self::Context<'_>, buf: &[u8]) -> Self::Return<Result<usize>>;
    fn flush(&mut self, cx: &mut Self::Context<'_>) -> Self::Return<Result<()>>;
    fn close(&mut self, cx: &mut Self::Context<'_>) -> Self::Return<Result<()>>;
}

impl<S> StreamExt<S, false> for Stream<S>
where
    S: std::io::Read + std::io::Write,
{
    type Context<'a> = ();
    type Return<T> = T;

    fn read(&mut self, _cx: &mut (), buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }

    fn write(&mut self, _cx: &mut (), buf: &[u8]) -> Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self, _cx: &mut ()) -> Result<()> {
        self.0.flush()
    }

    fn close(&mut self, _cx: &mut ()) -> Result<()> {
        Ok(())
    }
}

impl<S> std::io::Read for Stream<S>
where
    S: std::io::Read + std::io::Write,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        StreamExt::read(self, &mut (), buf)
    }
}

impl<S> std::io::Write for Stream<S>
where
    S: std::io::Read + std::io::Write,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        StreamExt::write(self, &mut (), buf)
    }

    fn flush(&mut self) -> Result<()> {
        StreamExt::flush(self, &mut ())
    }
}

impl<S> StreamExt<S, true> for Stream<S>
where
    S: futures::AsyncRead + futures::AsyncWrite + Unpin,
{
    type Context<'a> = Context<'a>;
    type Return<T> = Poll<T>;

    fn read(&mut self, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<Result<usize>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }

    fn write(&mut self, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn flush(&mut self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn close(&mut self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.0).poll_close(cx)
    }
}

impl<S> futures::AsyncRead for Stream<S>
where
    S: futures::AsyncRead + futures::AsyncWrite + Unpin,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        Stream::read(self.get_mut(), cx, buf)
    }
}

impl<S> futures::AsyncWrite for Stream<S>
where
    S: futures::AsyncRead + futures::AsyncWrite + Unpin,
{
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        Stream::write(self.get_mut(), cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Stream::flush(self.get_mut(), cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Stream::close(self.get_mut(), cx)
    }
}
