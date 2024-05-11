use super::ripemd160::CoreRipemd160Hasher;
use super::sha256::CoreSha256Hasher;
use crate::hash::base_types::hash160::Hash160;
use crate::hash::base_types::hash256::Hash256;

pub fn btc_hash256(data: &[u8]) -> Hash256 {
    CoreSha256Hasher::hash_bytes(&CoreSha256Hasher::hash_bytes(data).0)
}

pub fn btc_hash160(data: &[u8]) -> Hash160 {
    CoreRipemd160Hasher::hash_bytes(&CoreSha256Hasher::hash_bytes(data).0)
}
