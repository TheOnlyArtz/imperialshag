
use tokio::{
    net::TcpStream,
    io::{AsyncWriteExt}
};

pub enum SocketState {
    Handshake(HandshakeState),
    Operational
}

pub enum HandshakeState {
    ServerHello, // RSA public key
    ClientHello, // AES256 encrypted with the public key
}

pub struct SocketStream {
    pub stream: TcpStream,
    state: SocketState,
}

impl SocketStream {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            state: SocketState::Handshake(HandshakeState::ClientHello)
        }
    }

    pub async fn consume_message(&self) -> Result<(Vec<u8>, usize), ()> {
        let mut data = vec![0; 1024];

        match self.stream.try_read(&mut data) {
            Err(_) => return Err(()),
            Ok(n_bytes) => {
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

    pub async fn write_msg(&mut self, msg: &Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        self.stream.write_all(msg).await?; // TODO: Maybe error handling like broken pipes.

        Ok(())
    }
}