
use tokio::{
    net::TcpStream,
    // io::{AsyncRead, AsyncWrite, AsyncBufReadExt, BufReader}
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
        match &self.state { 
            SocketState::Handshake(handshake_state) => {
                match handshake_state {
                    ClientHello => {

                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }
}
// impl SocketStream<TcpStream> {
//     pub async fn from_connection(connection: TcpStream) -> Self {
//         SocketStream::new(connection)
//     }
// }

// impl<S: AsyncRead + AsyncWrite + Unpin> SocketStream<S> {
//     pub fn new(stream: S) -> Self {
//         Self {
//             reader: BufReader::new(stream),
//         }
//     }

//     pub async fn consume_message(&mut self) -> Result<(Vec<u8>, usize), Box<dyn std::error::Error>> {
//         let mut data = vec![0; 1024];

//         match self.
//     }
// }