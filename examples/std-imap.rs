use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

use rust_sandbox::Stream;

fn main() {
    let tcp_stream = TcpStream::connect(("posteo.de", 143)).unwrap();
    let stream = Stream::new(tcp_stream);
    println!("connected");

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    println!("read: {line:?}");

    let mut stream = reader.into_inner();
    let line = "A1 CAPABILITY\r\n";
    println!("write: {line:?}");
    stream.write_all(line.as_bytes()).unwrap();

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    println!("capability: {line:?}");

    println!("disconnected");
}
