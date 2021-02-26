use tokio::net::{TcpListener, TcpStream};
use tokio::io::Interest;
use std::io;

async fn process_socket(socket: TcpStream) -> io::Result<()> {
    // A loop which reads messages from the CnC
    loop {
        let stream_ready = socket.ready(Interest::READABLE | Interest::WRITABLE).await?;

        if stream_ready.is_readable() {
            let mut data = vec![0; 1024];

            match socket.try_read(&mut data) {
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                Err(e) => return Err(e.into()),
                Ok(n_bytes) => {
                    if n_bytes == 0 {
                        println!("Agent closed connection !");
                        return Ok(())
                    } 
                    println!("{}", String::from_utf8(data).unwrap());
                }
            }
        }
    }
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