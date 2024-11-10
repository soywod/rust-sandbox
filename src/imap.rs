use std::{collections::VecDeque, io::Result, sync::Arc};

use rustls_platform_verifier::ConfigVerifierExt;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};

#[derive(Clone, Debug)]
pub enum ImapEffect {
    ConnectPlain(String, u16),
    ConnectTls(String),
    Disconnect,
    ReadLine,
    WriteLine(String),
}

#[derive(Clone, Debug, Default)]
pub struct ImapState {
    effects: VecDeque<ImapEffect>,
}

impl ImapState {
    pub fn connect_plain(&mut self, host: impl ToString, port: u16) {
        self.effects
            .push_back(ImapEffect::ConnectPlain(host.to_string(), port));
    }

    pub fn connect_tls(&mut self, host: impl ToString) {
        self.effects
            .push_back(ImapEffect::ConnectTls(host.to_string()));
    }

    pub fn read_line(&mut self) {
        self.effects.push_back(ImapEffect::ReadLine);
    }

    pub fn write_line(&mut self, line: impl ToString) {
        let mut line = line.to_string();
        line.push_str("\r\n");
        self.effects.push_back(ImapEffect::WriteLine(line));
    }

    pub fn disconnect(&mut self) {
        self.effects.push_back(ImapEffect::Disconnect);
    }
}

impl Iterator for ImapState {
    type Item = ImapEffect;

    fn next(&mut self) -> Option<Self::Item> {
        self.effects.pop_front()
    }
}

pub struct ImapSafeTls;

impl ImapSafeTls {
    pub fn start(host: impl ToString, port: u16) -> ImapState {
        let mut state = ImapState::default();

        state.connect_plain(host.to_string(), port);

        state.read_line();
        state.write_line("A STARTTLS");
        state.read_line();

        state.connect_tls(host.to_string());
        state.write_line("B CAPABILITY");
        state.read_line();

        state.disconnect();

        state
    }
}

pub struct TokioImapSafeTls;

impl TokioImapSafeTls {
    pub async fn start(host: impl ToString, port: u16) -> Result<()> {
        let mut tcp_reader = None::<BufReader<ReadHalf<tokio::net::TcpStream>>>;
        let mut tcp_writer = None::<WriteHalf<tokio::net::TcpStream>>;

        let mut tls_enabled = false;
        let mut tls_reader =
            None::<BufReader<ReadHalf<tokio_rustls::client::TlsStream<tokio::net::TcpStream>>>>;
        let mut tls_writer =
            None::<WriteHalf<tokio_rustls::client::TlsStream<tokio::net::TcpStream>>>;

        let mut output = String::new();

        for effect in ImapSafeTls::start(host, port) {
            match effect {
                ImapEffect::ConnectPlain(host, port) => {
                    println!("connecting to {host}:{port} using plain TCP…");
                    let stream = tokio::net::TcpStream::connect((host, port)).await?;
                    let (r, w) = tokio::io::split(stream);
                    tcp_reader = Some(BufReader::new(r));
                    tcp_writer = Some(w);
                    println!("connected!");
                }
                ImapEffect::ConnectTls(host) => {
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
                ImapEffect::Disconnect => {
                    println!("disconnecting…");
                    if let Some(mut stream) = tcp_writer.take() {
                        tokio::io::AsyncWriteExt::shutdown(&mut stream).await?;
                    }
                    if let Some(mut stream) = tls_writer.take() {
                        tokio::io::AsyncWriteExt::shutdown(&mut stream).await?;
                    }
                }
                ImapEffect::ReadLine if tls_enabled => {
                    if let Some(reader) = tls_reader.as_mut() {
                        output.clear();
                        reader.read_line(&mut output).await?;
                        println!("read line from TLS: {output:?}");
                    }
                }
                ImapEffect::ReadLine => {
                    if let Some(reader) = tcp_reader.as_mut() {
                        output.clear();
                        reader.read_line(&mut output).await?;
                        println!("read line from TCP: {output:?}");
                    }
                }
                ImapEffect::WriteLine(buf) if tls_enabled => {
                    if let Some(writer) = tls_writer.as_mut() {
                        println!("write line using TLS: {buf:?}");
                        writer.write(buf.as_bytes()).await?;
                    }
                }
                ImapEffect::WriteLine(buf) => {
                    if let Some(writer) = tcp_writer.as_mut() {
                        println!("write line using TCP: {buf:?}");
                        writer.write(buf.as_bytes()).await?;
                    }
                }
            }
        }

        Ok(())
    }
}
