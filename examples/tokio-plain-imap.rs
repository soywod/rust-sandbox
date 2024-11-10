use rust_sandbox::imap::TokioImapSafeTls;

#[tokio::main]
async fn main() {
    let port = match std::env::var("PORT") {
        Ok(port) => port.parse().expect("port should be u16"),
        Err(_) => 143,
    };

    TokioImapSafeTls::start("posteo.de", port).await.unwrap();
}
