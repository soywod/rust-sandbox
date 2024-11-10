use std::{collections::VecDeque, io::Result, sync::Arc};

use rustls_platform_verifier::ConfigVerifierExt;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};

#[derive(Clone, Debug)]
pub enum StreamEffect {
    Connect(String, u16),
    Upgrade(String),
    DiscardLine,
    ReadLine,
    WriteLine(String),
    Disconnect,
}

#[derive(Clone, Debug, Default)]
pub struct StreamState {
    effects: VecDeque<StreamEffect>,
}

impl StreamState {
    pub fn connect(&mut self, host: impl ToString, port: u16) {
        self.effects
            .push_back(StreamEffect::Connect(host.to_string(), port));
    }

    pub fn upgrade(&mut self, host: impl ToString) {
        self.effects
            .push_back(StreamEffect::Upgrade(host.to_string()));
    }

    pub fn discard_line(&mut self) {
        self.effects.push_back(StreamEffect::DiscardLine);
    }

    pub fn read_line(&mut self) {
        self.effects.push_back(StreamEffect::ReadLine);
    }

    pub fn write_line(&mut self, line: impl ToString) {
        let mut line = line.to_string();
        line.push_str("\r\n");
        self.effects.push_back(StreamEffect::WriteLine(line));
    }

    pub fn disconnect(&mut self) {
        self.effects.push_back(StreamEffect::Disconnect);
    }
}

impl Iterator for StreamState {
    type Item = StreamEffect;

    fn next(&mut self) -> Option<Self::Item> {
        self.effects.pop_front()
    }
}

pub struct TokioRustlsStreamIo;

impl TokioRustlsStreamIo {
    pub async fn run(state: StreamState) -> Result<()> {
        let mut tcp_reader = None::<BufReader<ReadHalf<tokio::net::TcpStream>>>;
        let mut tcp_writer = None::<WriteHalf<tokio::net::TcpStream>>;

        let mut tls_enabled = false;
        let mut tls_reader =
            None::<BufReader<ReadHalf<tokio_rustls::client::TlsStream<tokio::net::TcpStream>>>>;
        let mut tls_writer =
            None::<WriteHalf<tokio_rustls::client::TlsStream<tokio::net::TcpStream>>>;

        let mut output = String::new();

        for effect in state {
            match effect {
                StreamEffect::Connect(host, port) => {
                    println!("connecting to {host}:{port} using plain TCP…");
                    let stream = tokio::net::TcpStream::connect((host, port)).await?;
                    let (r, w) = tokio::io::split(stream);
                    tcp_reader = Some(BufReader::new(r));
                    tcp_writer = Some(w);
                    println!("connected!");
                }
                StreamEffect::Upgrade(host) => {
                    let Some(reader) = tcp_reader.take() else {
                        continue;
                    };

                    let Some(writer) = tcp_writer.take() else {
                        continue;
                    };

                    let tcp_stream = reader.into_inner().unsplit(writer);

                    println!("connecting to {host} using TLS…");

                    let tls_config =
                        Arc::new(tokio_rustls::rustls::ClientConfig::with_platform_verifier());
                    let tls_connector = tokio_rustls::TlsConnector::from(tls_config);
                    let tls_stream = tls_connector
                        .connect(host.to_owned().try_into().unwrap(), tcp_stream)
                        .await
                        .unwrap();
                    let (r, w) = tokio::io::split(tls_stream);

                    tls_reader = Some(BufReader::new(r));
                    tls_writer = Some(w);
                    tls_enabled = true;
                    println!("connected!");
                }
                StreamEffect::DiscardLine if tls_enabled => {
                    if let Some(reader) = tls_reader.as_mut() {
                        let mut output = String::new();
                        reader.read_line(&mut output).await?;
                        for line in output.lines() {
                            println!("discard line from TLS: {line:?}");
                        }
                        output.clear();
                    }
                }
                StreamEffect::DiscardLine => {
                    if let Some(reader) = tcp_reader.as_mut() {
                        let mut output = String::new();
                        reader.read_line(&mut output).await?;
                        for line in output.lines() {
                            println!("discard line from TCP: {line:?}");
                        }
                        output.clear();
                    }
                }
                StreamEffect::ReadLine if tls_enabled => {
                    if let Some(reader) = tls_reader.as_mut() {
                        output.clear();
                        reader.read_line(&mut output).await?;
                        println!("read line from TLS: {output:?}");
                    }
                }
                StreamEffect::ReadLine => {
                    if let Some(reader) = tcp_reader.as_mut() {
                        output.clear();
                        reader.read_line(&mut output).await?;
                        println!("read line from TCP: {output:?}");
                    }
                }
                StreamEffect::WriteLine(buf) if tls_enabled => {
                    if let Some(writer) = tls_writer.as_mut() {
                        println!("write line using TLS: {buf:?}");
                        writer.write(buf.as_bytes()).await?;
                    }
                }
                StreamEffect::WriteLine(buf) => {
                    if let Some(writer) = tcp_writer.as_mut() {
                        println!("write line using TCP: {buf:?}");
                        writer.write(buf.as_bytes()).await?;
                    }
                }
                StreamEffect::Disconnect => {
                    println!("disconnecting…");
                    if let Some(mut stream) = tcp_writer.take() {
                        tokio::io::AsyncWriteExt::shutdown(&mut stream).await?;
                    }
                    if let Some(mut stream) = tls_writer.take() {
                        tokio::io::AsyncWriteExt::shutdown(&mut stream).await?;
                    }
                }
            }
        }

        Ok(())
    }
}
