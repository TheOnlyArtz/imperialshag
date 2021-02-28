use sodiumoxide::crypto::aead::{self, Key, Nonce};
extern crate openssl;
use openssl::rsa::{Padding, Rsa};

const PUBLIC_KEY: &str = r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA11wjxztHzpnl1jHpK4GR
WIOHR/98I8s6eXEqduEo/LXQ5fcGJP8MoTsCJmCNKp0rvw2akGZbedHrYm/ZzdIx
XK3RHyMlxnZLgmhbPRAqbgcK6Ocs9H/CfJ29Yyd5PWdrJVPkMe3HPWv3QxenlNSE
A6jsxgcFstiPpoDBjSaIjfmqJqZzKSmAAmqId4eS3KMyqOd1iZpKBBphPhYcVOJo
A/7N5gYVLC0cYNYO9FEo+h4ibTcX3+P0gt/sriY7qpGwRyj4adM/QrjNfy9SpuNE
kDZEb1MLidGGdA7UakC8klksYXmnsdOvtlhRdSEfDfoubuB0kg+rWapYiJ9jXRBS
WQIDAQAB
-----END PUBLIC KEY-----
"#; // It's an open source project so meh...

pub fn decrypt_from_aes(msg: Vec<u8>, key: &Key, nonce: &Nonce) -> Vec<u8> {
    let c = aead::open(&msg, None, nonce, key);
    c.unwrap()
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
