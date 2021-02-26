use tokio::net::{TcpListener, TcpStream};
use tokio::io::Interest;
use std::io;

use crate::socket::{SocketState, SocketStream};

async fn process_socket(socket: TcpStream) -> io::Result<()> {
    // A loop which reads messages from the CnC
    let socket = SocketStream::new(socket);

    loop {
        let stream_ready = socket.stream.ready(Interest::READABLE | Interest::WRITABLE).await?;

        if stream_ready.is_readable() {
            
            let msg = socket.consume_message().await;

            if let Ok((msg, n_bytes)) = msg {
                if n_bytes == 0 {
                    break
                }
                
                socket.handle_msg(msg).await;
            }
        }
    }

    Ok(())
}

pub async fn start_cnc_server(ip: &str, port: u16) -> io::Result<()> {
    let listener = TcpListener::bind(&format!("{}:{}", ip, port)).await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            process_socket(socket).await;
        }).await?;
    }
}