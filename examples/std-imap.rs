use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::Arc,
};

use rust_sandbox::Stream;
use rustls::{ClientConfig, ClientConnection, StreamOwned};
use rustls_platform_verifier::ConfigVerifierExt;

fn main() {
    const HOST: &str = "posteo.de";

    println!("connecting using TCP…");
    let tcp_stream = TcpStream::connect((HOST, 143)).unwrap();
    let mut tcp_stream = Stream::from(tcp_stream);

    println!("preparing for STARTTLS…");
    tcp_stream.prepare_imap_starttls().unwrap();

    println!("connecting using TLS…");
    let tls_config = Arc::new(ClientConfig::with_platform_verifier());
    let tls_connection = ClientConnection::new(tls_config, HOST.to_owned().try_into().unwrap());
    let mut tls_stream = StreamOwned::new(tls_connection.unwrap(), tcp_stream);

    println!("asking for capabilities…");
    tls_stream.write_all(b"A2 CAPABILITY\r\n").unwrap();

    let mut line = String::new();
    BufReader::new(tls_stream).read_line(&mut line).unwrap();
    println!("capabilities: {line:?}");
}
