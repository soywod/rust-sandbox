pub mod starttls;
pub mod stream;

use std::{
    io::Result,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Stream<S, const ASYNC: bool>(S);

impl<S, const ASYNC: bool> Stream<S, ASYNC> {
    pub fn new(stream: S) -> Self {
        Self(stream)
    }
}

pub trait StreamExt<const ASYNC: bool> {
    type Stream;
    type Context<'a>;
    type Return<T>;

    fn read(&mut self, cx: &mut Self::Context<'_>, buf: &mut [u8]) -> Self::Return<usize>;

    fn write(&mut self, cx: &mut Self::Context<'_>, buf: &[u8]) -> Self::Return<usize>;
    fn flush(&mut self, cx: &mut Self::Context<'_>) -> Self::Return<()>;
    fn close(&mut self, cx: &mut Self::Context<'_>) -> Self::Return<()>;
}

impl<S> StreamExt<false> for Stream<S, false>
where
    S: std::io::Read + std::io::Write,
{
    type Stream = S;
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

impl<S> std::io::Read for Stream<S, false>
where
    S: std::io::Read + std::io::Write,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        StreamExt::read(self, &mut (), buf)
    }
}

impl<S> std::io::Write for Stream<S, false>
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

impl<S: std::io::Read + std::io::Write> Stream<S, false> {
    pub fn prepare_imap_starttls(&mut self) -> Result<()> {
        self.prepare_starttls("A1 STARTTLS\r\n")
    }

    pub fn prepare_starttls(&mut self, cmd: &str) -> Result<()> {
        self.skip_line()?;
        println!("write line: {cmd:?}");
        std::io::Write::write_all(self, cmd.as_bytes())?;
        self.skip_line()?;
        Ok(())
    }

    pub fn skip_line(&mut self) -> Result<()> {
        let mut cr = false;
        let mut buf = [0; 1];

        loop {
            std::io::Read::read_exact(self, &mut buf)?;
            println!("skip char: {:?}", buf[0] as char);

            match buf[0] {
                b'\r' => {
                    cr = true;
                    continue;
                }
                b'\n' if cr => {
                    break Ok(());
                }
                _ => {
                    continue;
                }
            };
        }
    }
}

impl<S> StreamExt<true> for Stream<S, true>
where
    S: futures::AsyncRead + futures::AsyncWrite + Unpin,
{
    type Stream = S;
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

impl<S> futures::AsyncRead for Stream<S, true>
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

impl<S> futures::AsyncWrite for Stream<S, true>
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

impl<S: futures::AsyncRead + futures::AsyncWrite + Unpin> Stream<S, true> {
    pub async fn prepare_imap_starttls(&mut self) -> Result<()> {
        self.prepare_starttls("A1 STARTTLS\r\n").await
    }

    pub async fn prepare_starttls(&mut self, cmd: &str) -> Result<()> {
        self.skip_line().await?;
        println!("write line: {cmd:?}");
        futures::AsyncWriteExt::write_all(self, cmd.as_bytes()).await?;
        self.skip_line().await?;
        Ok(())
    }

    pub async fn skip_line(&mut self) -> Result<()> {
        let mut cr = false;
        let mut buf = [0; 1];

        loop {
            futures::AsyncReadExt::read_exact(self, &mut buf).await?;
            println!("skip char: {:?}", buf[0] as char);

            match buf[0] {
                b'\r' => {
                    cr = true;
                    continue;
                }
                b'\n' if cr => {
                    break Ok(());
                }
                _ => {
                    continue;
                }
            };
        }
    }
}
