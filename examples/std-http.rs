use rust_sandbox::StdHttp;

fn main() {
    let response = StdHttp::get_plain_root("www.rust-lang.org").unwrap();
    println!("response: {response:?}");
}
