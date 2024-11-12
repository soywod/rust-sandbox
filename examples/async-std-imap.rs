use std::sync::Arc;

use async_std::{io::BufReader, net::TcpStream};
use futures::{AsyncBufReadExt, AsyncWriteExt};
use futures_rustls::TlsConnector;
use rust_sandbox::Stream;
use rustls::ClientConfig;
use rustls_platform_verifier::ConfigVerifierExt;

#[async_std::main]
async fn main() {
    const HOST: &str = "posteo.de";

    println!("connecting using TCP…");
    let tcp_stream = TcpStream::connect((HOST, 143)).await.unwrap();
    let mut tcp_stream = Stream::from(tcp_stream);

    println!("preparing for STARTTLS…");
    tcp_stream.prepare_imap_starttls().await.unwrap();

    println!("connecting using TLS…");
    let tls_config = Arc::new(ClientConfig::with_platform_verifier());
    let tls_connector = TlsConnector::from(tls_config);
    let mut tls_stream = tls_connector
        .connect(HOST.to_owned().try_into().unwrap(), tcp_stream)
        .await
        .unwrap();

    println!("asking for capabilities…");
    tls_stream.write_all(b"A2 CAPABILITY\r\n").await.unwrap();

    let mut line = String::new();
    BufReader::new(tls_stream)
        .read_line(&mut line)
        .await
        .unwrap();
    println!("capabilities: {line:?}");
}
