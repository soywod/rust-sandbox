pub mod starttls;
pub mod stream;

use std::{
    future::Future,
    io::{Error, ErrorKind, Result},
    pin::{pin, Pin},
    task::{ready, Context, Poll},
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

    fn read(&mut self, cx: &mut Self::Context<'_>, buf: &mut [u8]) -> Self::Return<usize>;

    fn write(&mut self, cx: &mut Self::Context<'_>, buf: &[u8]) -> Self::Return<usize>;
    fn flush(&mut self, cx: &mut Self::Context<'_>) -> Self::Return<()>;
    fn close(&mut self, cx: &mut Self::Context<'_>) -> Self::Return<()>;
}

impl<S> StreamExt<S, false> for Stream<S>
where
    S: std::io::Read + std::io::Write,
{
    type Context<'a> = ();
    type Return<T> = Result<T>;

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
    type Return<T> = Poll<Result<T>>;

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

pub trait PrepareStartTlsExt<S, const ASYNC: bool> {
    type Context<'a>;
    type Return<T>;

    fn prepare(&mut self, cx: &mut Self::Context<'_>) -> Self::Return<Stream<S>>;
}

pub struct PrepareStartTls<S> {
    stream: Option<Stream<S>>,
    starttls_command: String,
}

impl<S> PrepareStartTls<S> {
    pub fn new(stream: Stream<S>, starttls_command: impl ToString) -> Self {
        Self {
            stream: Some(stream),
            starttls_command: starttls_command.to_string(),
        }
    }

    pub fn imap(stream: Stream<S>) -> Self {
        Self::new(stream, "A1 STARTTLS\r\n")
    }
}

impl<S> PrepareStartTlsExt<S, false> for PrepareStartTls<S>
where
    S: std::io::Read + std::io::Write,
{
    type Context<'a> = ();
    type Return<T> = Result<T>;

    fn prepare(&mut self, _cx: &mut ()) -> Result<Stream<S>> {
        use std::io::{BufRead, BufReader};

        let Some(stream) = self.stream.take() else {
            return Err(Error::new(
                ErrorKind::OutOfMemory,
                "stream already prepared",
            ));
        };

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        println!("read: {line:?}");
        line.clear();

        let mut stream = reader.into_inner();
        println!("write: {:?}", self.starttls_command);
        stream.0.write_all(self.starttls_command.as_bytes())?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        println!("read: {line:?}");
        line.clear();

        Ok(reader.into_inner())
    }
}

impl<S> PrepareStartTlsExt<S, true> for PrepareStartTls<S>
where
    S: futures::AsyncRead + futures::AsyncWrite + Unpin,
{
    type Context<'a> = Context<'a>;
    type Return<T> = Poll<Result<T>>;

    fn prepare(&mut self, cx: &mut Context<'_>) -> Poll<Result<Stream<S>>> {
        // FIXME: cannot make it work, future blocks on Pending
        unimplemented!()
    }
}

impl<S> Future for PrepareStartTls<S>
where
    S: futures::AsyncRead + futures::AsyncWrite + Unpin,
{
    type Output = Result<Stream<S>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = Pin::into_inner(self);

        let Some(stream) = this.stream.take() else {
            return Poll::Ready(Err(Error::new(
                ErrorKind::OutOfMemory,
                "stream already prepared",
            )));
        };

        pin!(prepare_imap_starttls(stream)).poll(cx)
    }
}

pub async fn prepare_imap_starttls<S>(stream: Stream<S>) -> Result<Stream<S>>
where
    S: futures::AsyncRead + futures::AsyncWrite + Unpin,
{
    use futures::{
        io::{AsyncBufReadExt, BufReader},
        AsyncWriteExt,
    };

    let mut reader = BufReader::new(stream);

    let mut line = String::new();
    reader.read_line(&mut line).await?;
    println!("read: {line:?}");

    let mut stream = reader.into_inner();
    println!("write STARTTLS");
    stream.write_all(b"A1 STARTTLS\r\n").await?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    println!("read: {line:?}");
    line.clear();

    Ok(reader.into_inner())
}
