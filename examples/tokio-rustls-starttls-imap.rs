use rust_sandbox::{starttls::StartTlsProvider, stream::TokioRustlsStreamIo};

#[tokio::main]
async fn main() {
    let host = std::env::var("HOST").expect("HOST should be defined");
    let port: u16 = std::env::var("PORT")
        .expect("PORT should be defined")
        .parse()
        .expect("PORT should be an unsigned integer");

    let starttls_provider = StartTlsProvider::new(host, port).imap();

    TokioRustlsStreamIo::run(starttls_provider).await.unwrap();
}
