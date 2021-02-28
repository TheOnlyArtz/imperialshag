use sodiumoxide::crypto::aead::{self, Key, Nonce};

extern crate openssl;
use openssl::rsa::{Padding, Rsa};

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
}

pub fn decrypt_with_rsa(msg: Vec<u8>, key: Vec<u8>) -> Result<Vec<u8>, openssl::error::ErrorStack> {
    let rsa = Rsa::private_key_from_pem(&key).unwrap();
    let mut decrypted: Vec<u8> = vec![0; rsa.size() as usize];
    let res = rsa
        .private_decrypt(&msg, &mut decrypted, Padding::PKCS1);

    match res {
        Ok(_) => Ok(decrypted),
        Err(e) => Err(e)
    }
}
