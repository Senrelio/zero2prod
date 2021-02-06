use std::net::TcpListener;

use zero2prod::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:9001").unwrap();
    run(listener)?.await
}
