use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{digest::CtOutput, Sha256};
use zeroize::Zeroize;

use super::{
    message_key::{MessageKey, RemoteMessageKey},
    ratchet::RatchetPublicKey,
};

const MESSAGE_KEY_SEED: &[u8; 1] = b"\x01";
const ADVANCEMENT_SEED: &[u8; 1] = b"\x02";

fn expand_chain_key(key: &[u8; 32]) -> [u8; 32] {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(key).expect("Can't create HmacSha256 from the key");
    mac.update(MESSAGE_KEY_SEED);

    let output = mac.finalize().into_bytes();

    let mut key = [0u8; 32];
    key.copy_from_slice(output.as_slice());

    key
}

fn advance(key: &[u8; 32]) -> CtOutput<Hmac<Sha256>> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .expect("Coulnd't create a valid Hmac object to advance the ratchet");
    mac.update(ADVANCEMENT_SEED);

    mac.finalize()
}

#[derive(Clone, Zeroize, Serialize, Deserialize)]
pub(super) struct ChainKey {
    key: [u8; 32],
    index: u64,
}

impl Drop for ChainKey {
    fn drop(&mut self) {
        self.key.zeroize()
    }
}

#[derive(Clone, Zeroize, Serialize, Deserialize)]
pub(super) struct RemoteChainKey {
    key: [u8; 32],
    index: u64,
}

impl Drop for RemoteChainKey {
    fn drop(&mut self) {
        self.key.zeroize()
    }
}

impl RemoteChainKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self { key: bytes, index: 0 }
    }

    pub fn chain_index(&self) -> u64 {
        self.index
    }

    pub fn from_bytes_and_index(bytes: [u8; 32], index: u32) -> Self {
        Self { key: bytes, index: index.into() }
    }

    pub fn advance(&mut self) {
        let output = advance(&self.key).into_bytes();
        self.key.copy_from_slice(output.as_slice());
        self.index += 1;
    }

    pub fn create_message_key(&mut self) -> RemoteMessageKey {
        let key = expand_chain_key(&self.key);
        let message_key = RemoteMessageKey::new(key, self.index);

        self.advance();

        message_key
    }
}

impl ChainKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self { key: bytes, index: 0 }
    }

    pub fn from_bytes_and_index(bytes: [u8; 32], index: u32) -> Self {
        Self { key: bytes, index: index.into() }
    }

    pub fn advance(&mut self) {
        let output = advance(&self.key).into_bytes();
        self.key.copy_from_slice(output.as_slice());
        self.index += 1;
    }

    pub fn create_message_key(&mut self, ratchet_key: RatchetPublicKey) -> MessageKey {
        let key = expand_chain_key(&self.key);
        let message_key = MessageKey::new(key, ratchet_key, self.index);

        self.advance();

        message_key
    }
}