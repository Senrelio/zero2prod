use std::net::TcpListener;

use zero2prod::configuration::get_configuration;
use zero2prod::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("failed to read configuration.");
    let addr = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(addr).unwrap();
    run(listener)?.await
}
