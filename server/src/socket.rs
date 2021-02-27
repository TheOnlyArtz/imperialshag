use tokio::{
    net::TcpStream,
    io::{AsyncWriteExt},
    sync::{Mutex}
};

use std::sync::Arc;

#[derive(Debug)]
pub enum SocketState {
    Handshake(HandshakeState),
    Operational
}

#[derive(Debug)]
pub enum HandshakeState {
    ServerHello, // RSA public key
    ClientHello, // AES256 encrypted with the public key
}

#[derive(Debug)]
pub struct SocketStream {
    pub stream: Arc<Mutex<TcpStream>>,
    state: SocketState,
}

impl SocketStream {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: Arc::new(Mutex::new(stream)),
            state: SocketState::Handshake(HandshakeState::ClientHello)
        }
    }

    pub async fn consume_message(&self) -> Result<(Vec<u8>, usize), std::io::Error> {
        let mut data = vec![0; 1024];
        let my_stream = Arc::clone(&self.stream);
        let stream_lock = my_stream.lock().await;

        match stream_lock.try_read(&mut data) {
            Err(e) => {
                return Err(e)
            },
            Ok(n_bytes) => {
                println!("{}", n_bytes);
                if n_bytes == 0 {
                    println!("Agent closed connection !");
                    return Ok((vec![0u8], 0));
                }
                
                Ok((data, n_bytes))
            },
        }
    }

    pub async fn handle_msg(&self, msg: Vec<u8>) {
        println!("{:?}", msg);
        match &self.state { 
            SocketState::Handshake(handshake_state) => {
                match handshake_state {
                    HandshakeState::ClientHello => {

                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    pub async fn write_msg(&self, msg: &Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let my_stream = Arc::clone(&self.stream);
        let mut stream_lock = my_stream.lock().await;
        stream_lock.write_all(msg).await?; // TODO: Maybe error handling like broken pipes.

        Ok(())
    }
}