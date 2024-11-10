pub mod stream;

use std::io::Result;

use stream::{StreamEffect, StreamState};

/// Basic HTTP sans I/O service.
/// Should be used lib side.
pub struct Http;

impl Http {
    pub fn get_plain_root(host: impl ToString) -> StreamState {
        let mut stream = StreamState::default();

        stream.connect(host.to_string(), 80);

        let request = format!("GET / HTTP/1.0\r\nHost: {}\r\n\r\n", host.to_string());
        stream.write_all(request);

        stream.read_to_string();

        stream.disconnect();

        stream
    }
}

/// Standard I/O connector for [`Http`] service.
/// Should be used app side.
pub struct StdHttp;

impl StdHttp {
    pub fn get_plain_root(host: impl ToString) -> Result<String> {
        let mut stream = None::<std::net::TcpStream>;
        let mut output = String::new();

        for effect in Http::get_plain_root(host) {
            match effect {
                StreamEffect::Connect(host, port) => {
                    stream = Some(std::net::TcpStream::connect((host, port))?)
                }
                StreamEffect::Disconnect => {
                    if let Some(stream) = stream.take() {
                        stream.shutdown(std::net::Shutdown::Both)?;
                    }
                }
                StreamEffect::ReadToString => {
                    if let Some(stream) = stream.as_mut() {
                        output.clear();
                        std::io::Read::read_to_string(stream, &mut output)?;
                    }
                }
                StreamEffect::WriteAll(buf) => {
                    if let Some(stream) = stream.as_mut() {
                        std::io::Write::write_all(stream, &buf)?;
                    }
                }
            }
        }

        Ok(output)
    }
}

/// Tokio-based async I/O connector for [`Http`] service.
/// Should be used app side.
pub struct TokioHttp;

impl TokioHttp {
    pub async fn get_plain_root(host: impl ToString) -> Result<String> {
        let mut stream = None::<tokio::net::TcpStream>;
        let mut output = String::new();

        for effect in Http::get_plain_root(host) {
            match effect {
                StreamEffect::Connect(host, port) => {
                    stream = Some(tokio::net::TcpStream::connect((host, port)).await?)
                }
                StreamEffect::Disconnect => {
                    if let Some(mut stream) = stream.take() {
                        tokio::io::AsyncWriteExt::shutdown(&mut stream).await?;
                    }
                }
                StreamEffect::ReadToString => {
                    if let Some(stream) = stream.as_mut() {
                        output.clear();
                        tokio::io::AsyncReadExt::read_to_string(stream, &mut output).await?;
                    }
                }
                StreamEffect::WriteAll(buf) => {
                    if let Some(stream) = stream.as_mut() {
                        tokio::io::AsyncWriteExt::write_all(stream, &buf).await?;
                    }
                }
            }
        }

        Ok(output)
    }
}
