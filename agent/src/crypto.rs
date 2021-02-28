use sodiumoxide::crypto::aead::{self, Key, Nonce};
extern crate openssl;
use openssl::rsa::{Padding, Rsa};

const PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAvXOgxzv7L9+tyJb9Ghqp
FmFUmwboCuj4OVSwzbGIE3Im3H3lYdQsML7AtAj4kTggD452p9Q2IT7XKYYgnKGx
IKiEEcQooWg4IyWedIkQ66ZUli1nDgRZUh8S5HFu2TkyCanVLzZT/NpiLHuHzs8E
qd/9uQx/YfChhv23xuKZJfhdyHNc/Xh6o/LwSlv60hOOTdwft9FDkWCi3w3ktao1
6hXKFCjfS06gIDNWUwUUZTCI9EYrjpUXD9Jd7rd+c6UA59QW31qf5MTKMTv9IO/c
Vq7xmzO1AQpSX2fSxiRtfhzcmDaEgWhub4NOI70iaJ5bg2qx3wMdE2dOsfmC0Ljo
FwIDAQAB
-----END PUBLIC KEY-----";

pub fn decrypt_from_aes(msg: Vec<u8>, key: &Key, nonce: &Nonce) -> Result<Vec<u8>, ()> {
    let c = aead::open(&msg, None, nonce, key);
    c
}

pub fn encrypt_with_aes(msg: Vec<u8>, key: &Key, nonce: &Nonce) -> Vec<u8> {
    let c = aead::seal(&msg, None, &nonce, &key);

    c
}

pub fn generate_aes() -> (Key, Nonce) {
    let k = aead::gen_key(); // sodiumoxide::crypto::aead::Key
    let n = aead::gen_nonce(); // sodiumoxide::crypto::aead::Nonce

    (k, n)
}

pub fn encrypt_with_rsa(msg: Vec<u8>) -> Result<Vec<u8>, ()> {
    let rsa = Rsa::public_key_from_pem(PUBLIC_KEY.as_bytes());

    if rsa.is_err() {
        return Err(());
    }

    let rsa = rsa.unwrap();
    let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
    let _ = rsa.public_encrypt(&msg, &mut buf, Padding::PKCS1).unwrap();
    Ok(buf)
}
