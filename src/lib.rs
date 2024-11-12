pub mod starttls;
pub mod stream;

use std::{
    io::Result,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Stream<S, const ASYNC: bool>(S);

impl<S> From<S> for Stream<S, false>
where
    S: std::io::Read + std::io::Write,
{
    fn from(stream: S) -> Self {
        Self(stream)
    }
}

impl<S> std::io::Read for Stream<S, false>
where
    S: std::io::Read + std::io::Write,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }
}

impl<S> std::io::Write for Stream<S, false>
where
    S: std::io::Read + std::io::Write,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.0.flush()
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

impl<S> From<S> for Stream<S, true>
where
    S: futures::AsyncRead + futures::AsyncWrite + Unpin,
{
    fn from(stream: S) -> Self {
        Self(stream)
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
        Pin::new(&mut self.get_mut().0).poll_read(cx, buf)
    }
}

impl<S> futures::AsyncWrite for Stream<S, true>
where
    S: futures::AsyncRead + futures::AsyncWrite + Unpin,
{
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        Pin::new(&mut self.get_mut().0).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.get_mut().0).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.get_mut().0).poll_close(cx)
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
