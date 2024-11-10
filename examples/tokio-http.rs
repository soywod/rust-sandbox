use rust_sandbox::http::TokioHttp;

#[tokio::main]
async fn main() {
    let response = TokioHttp::get_plain_root("www.rust-lang.org")
        .await
        .unwrap();
    println!("response: {response:?}");
}
