use rsa::{RsaPrivateKey, RsaPublicKey, pkcs8::EncodePublicKey, rand_core::OsRng};
use rand::RngCore;

pub struct EncryptionKeyPair {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
    pub verify_token: Vec<u8>,
}

impl EncryptionKeyPair {
    pub fn new() -> Self {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 1024).expect("failed to generate private key");
        let public_key = RsaPublicKey::from(&private_key);

        let mut verify_token = vec![0u8; 16];
        rand::thread_rng().fill_bytes(&mut verify_token);

        Self {
            private_key,
            public_key,
            verify_token,
        }
    }

    pub fn public_key_der(&self) -> Vec<u8> {
        self.public_key.to_public_key_der().unwrap().as_bytes().to_vec()
    }

    pub fn private_key(&self) -> &RsaPrivateKey {
        &self.private_key
    }

    pub fn verify_token(&self) -> &[u8] {
        &self.verify_token
    }
}