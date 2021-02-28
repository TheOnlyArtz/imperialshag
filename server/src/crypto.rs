use sodiumoxide::crypto::aead::{self, Key, Nonce};

extern crate openssl;
use openssl::rsa::{Rsa, Padding};

use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub fn decrypt_from_aes(msg: Vec<u8>, key: &Key, nonce: &Nonce) -> Vec<u8> {
    let c = aead::open(&msg, None, nonce, key);

    c.unwrap()
}

pub fn encrypt_with_aes(msg: Vec<u8>, key: &Key, nonce: &Nonce) -> Vec<u8> {
    let c = aead::seal(&msg, None, &nonce, &key);
    
    c
}

// private / public
pub async fn load_private_rsa(file_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path).await?;

    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;

    Ok(contents)
    // let rsa = Rsa::generate(1024).unwrap();
    // let private_key: Vec<u8> = rsa.private_key_to_pem().unwrap();
    // let public_key: Vec<u8> = rsa.public_key_to_pem().unwrap();

}