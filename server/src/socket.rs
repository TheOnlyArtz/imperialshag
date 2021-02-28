use sodiumoxide::crypto::aead::{Key, Nonce};

use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};

use crate::crypto;
use std::convert::TryInto;
use std::sync::Arc;

fn to_arr_32<T>(v: Vec<T>) -> [T; 32] {
    v.try_into().unwrap_or_else(|v: Vec<T>| {
        panic!("Expected a Vec of length {} but it was {}", 32, v.len())
    })
}

fn to_arr_24<T>(v: Vec<T>) -> [T; 24] {
    v.try_into().unwrap_or_else(|v: Vec<T>| {
        panic!("Expected a Vec of length {} but it was {}", 24, v.len())
    })
}

#[derive(Debug)]
pub enum SocketState {
    Handshake(HandshakeState),
    Operational,
}

#[derive(Debug)]
pub enum HandshakeState {
    ServerHello, // RSA public key
    ClientHello, // AES256 encrypted with the pubkeylic key
}

#[derive(Debug)]
pub struct SocketStream {
    pub stream: Arc<Mutex<TcpStream>>,
    pub aes_key: Option<Key>,
    pub aes_nonce: Option<Nonce>,
    pub state: SocketState,
}

impl SocketStream {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: Arc::new(Mutex::new(stream)),
            state: SocketState::Handshake(HandshakeState::ClientHello),
            aes_key: None,
            aes_nonce: None,
        }
    }

    /*
        TODO: Pass the length of the payload the agent sends in the first byte
        that way we can the first byte, and read to a fixed length, much better.
    */
    pub async fn consume_message(&self) -> Result<(Vec<u8>, usize), std::io::Error> {
        let mut data = vec![0; 4096];
        let my_stream = Arc::clone(&self.stream);
        let stream_lock = my_stream.lock().await;

        match stream_lock.try_read(&mut data) {
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
        // self.write_msg(&msg).await.unwrap();
        println!("Number of bytes {:?}", n_bytes);
        let msg = msg.get(..n_bytes).unwrap().to_vec();
        match &self.state {
            SocketState::Handshake(handshake_state) => match handshake_state {
                HandshakeState::ClientHello => {
                    let msg = crypto::decrypt_with_rsa(
                        msg,
                        crypto::load_private_rsa("private.pem").await.unwrap(),
                    );

                    let msg = msg.get(..n_bytes).unwrap().to_vec();
                    let msg = String::from_utf8(msg).unwrap();

                    let iter: Vec<&str> = msg.split(" ").collect();
                    let key = iter.get(0).unwrap();
                    let key = base64::decode(key).unwrap();
                    let nonce = iter.get(1).unwrap().trim_matches(char::from(0));
                    let nonce = base64::decode(nonce).unwrap();
                    let key = Key(to_arr_32(key));
                    let nonce = Nonce(to_arr_24(nonce));

                    self.aes_key = Some(key);
                    self.aes_nonce = Some(nonce);
                    self.state = SocketState::Operational;

                    self.send_handshake_ack().await;
                }
                _ => {}
            },
            SocketState::Operational => {
                let decrypted_msg = crypto::decrypt_from_aes(
                    msg.to_vec(),
                    self.aes_key.as_ref().unwrap(),
                    &self.aes_nonce.unwrap(),
                );

                let msg = String::from_utf8(decrypted_msg).unwrap();

                println!("New msg: {}", msg);
            }
        }
    }

    pub async fn write_msg(&self, msg: &Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let my_stream = Arc::clone(&self.stream);
        let mut stream_lock = my_stream.lock().await;
        stream_lock.write_all(msg).await?; // TODO: Maybe error handling like broken pipes.
        Ok(())
    }

    pub async fn send_handshake_ack(&self) {
        let encrypted = crypto::encrypt_with_aes(
            b"ACK".to_vec(),
            self.aes_key.as_ref().unwrap(),
            self.aes_nonce.as_ref().unwrap(),
        );

        self.write_msg(&encrypted).await.unwrap();
    }
}
