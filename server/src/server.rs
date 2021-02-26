use tokio::net::{TcpListener, TcpStream};
use std::io;

async fn process_socket(socket: TcpStream) {
    println!("{:?}", socket);
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