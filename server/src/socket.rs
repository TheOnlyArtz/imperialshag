use sodiumoxide::crypto::aead::{Key, Nonce};

use std::net::SocketAddr;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};

use crate::crypto;
use std::convert::TryInto;
use std::sync::Arc;

#[derive(Debug)]
pub enum SocketState {
    Handshake(HandshakeState),
    Operational,
}

#[derive(Debug)]
pub enum HandshakeState {
    ClientHello, // AES256 encrypted with the pubkeylic key
}

// SocketAddr just indicates which peer has been disconnected.
#[derive(Debug)]
pub enum HandleErrors {
    BadHandshakeFormat(SocketAddr),
    BadHandshakeRSA(SocketAddr),
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

    pub async fn handle_msg(&mut self, msg: Vec<u8>, n_bytes: usize) -> Result<(), HandleErrors> {
        // We want to ensure we read only the bytes we need
        // (more bytes than the actual message can cause huge problems with the encryption algorithms)
        // For example "{some_encrypted_data}\0\0\0" can't be decrypted the way "{some_decrypted_data}" does.
        let msg = msg.get(..n_bytes).unwrap().to_vec();

        match &self.state {
            SocketState::Handshake(handshake_state) => match handshake_state {
                HandshakeState::ClientHello => {
                    println!("Goes ClientHello, starting to process handshake");

                    // Messing with RSA is a matter of once in an agent runtime.
                    // we shouldn't really care about the expenses of the method
                    // this method may return an error, every encryption related
                    // error SHOULD be taken seriously and we can't let the connection / handshake
                    // go through. SHUTDOWN!
                    let msg = crypto::decrypt_with_rsa(
                        msg,
                        crypto::load_private_rsa("private.pem").await.unwrap(),
                    );

                    if msg.is_err() {
                        return Err(HandleErrors::BadHandshakeRSA(
                            self.stream.as_ref().lock().await.peer_addr().unwrap(),
                        ));
                    }

                    let msg = msg.unwrap().get(..n_bytes).unwrap().to_vec();
                    let msg = String::from_utf8(msg).unwrap();

                    let iter: Vec<&str> = msg.split(" ").collect();
                    if iter.len() != 2 {
                        return Err(HandleErrors::BadHandshakeFormat(
                            self.stream.as_ref().lock().await.peer_addr().unwrap(),
                        )); // Doesn't follow the format of "Key Nonce\{bunch of nulls probably}"
                    }

                    let key = iter.get(0).unwrap();
                    let key = base64::decode(key).unwrap();
                    let nonce = iter.get(1).unwrap().trim_matches(char::from(0));
                    let nonce = base64::decode(nonce).unwrap();
                    let key = Key(to_arr_32(key));
                    let nonce = Nonce(to_arr_24(nonce));

                    self.aes_key = Some(key);
                    self.aes_nonce = Some(nonce);
                    self.state = SocketState::Operational;

                    // This means everything went as it should
                    println!("Handshake was made successfully!");
                    println!("Sending ACK to the agent");
                    self.send_handshake_ack().await;
                }
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

        Ok(())
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
