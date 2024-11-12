use async_std::{io::BufReader, net::TcpStream};
use futures::{AsyncBufReadExt, AsyncWriteExt};
use rust_sandbox::Stream;

#[async_std::main]
async fn main() {
    let tcp_stream = TcpStream::connect(("posteo.de", 143)).await.unwrap();
    let stream = Stream::new(tcp_stream);
    println!("connected");

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    println!("read: {line:?}");

    let mut stream = reader.into_inner();
    let line = "A1 CAPABILITY\r\n";
    println!("write: {line:?}");
    stream.write_all(line.as_bytes()).await.unwrap();

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    println!("capability: {line:?}");

    println!("disconnected");
}
