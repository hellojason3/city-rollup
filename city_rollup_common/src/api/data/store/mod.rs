use city_common::binaryhelpers::bytes::CompressedPublicKey;
use city_crypto::hash::{
    base_types::{hash160::Hash160, hash256::Hash256},
    qhashout::QHashOut,
};
use kvq::traits::KVQSerializable;
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::PrimeField64},
    hash::hash_types::RichField,
};
use serde::{Deserialize, Serialize};

type F = GoldilocksField;

#[derive(Debug, Clone, Serialize, Deserialize, Copy, Hash, Eq, PartialEq)]
pub struct CityUserState {
    pub user_id: u64,
    pub balance: u64,
    pub nonce: u64,
    pub alt_0: u64,
    pub alt_1: u64,
    pub public_key: QHashOut<F>,
}
impl CityUserState {
    pub fn from_hash(user_id: u64, left: QHashOut<F>, right: QHashOut<F>) -> Self {
        Self {
            user_id,
            balance: left.0.elements[0].to_canonical_u64(),
            nonce: left.0.elements[1].to_canonical_u64(),
            alt_0: left.0.elements[2].to_canonical_u64(),
            alt_1: left.0.elements[3].to_canonical_u64(),
            public_key: right,
        }
    }
    pub fn new_user_with_public_key(user_id: u64, public_key: QHashOut<F>) -> Self {
        Self {
            user_id,
            balance: 0,
            nonce: 0,
            alt_0: 0,
            alt_1: 0,
            public_key,
        }
    }
    pub fn get_left_leaf(&self) -> QHashOut<F> {
        QHashOut::from_values(self.balance, self.nonce, self.alt_0, self.alt_1)
    }
    pub fn get_right_leaf(&self) -> QHashOut<F> {
        self.public_key
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Copy, Hash, Eq, PartialEq)]
pub struct CityL1Deposit {
    pub deposit_id: u64,
    pub checkpoint_id: u64,
    pub value: u64,
    pub txid: Hash256,
    pub public_key: CompressedPublicKey,
}
impl KVQSerializable for CityL1Deposit {
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        // 8 + 8 + 8 + 32 + 33 = 89 bytes
        let mut result = Vec::with_capacity(89);
        result.extend_from_slice(&self.checkpoint_id.to_be_bytes());
        result.extend_from_slice(&self.deposit_id.to_be_bytes());
        result.extend_from_slice(&self.value.to_be_bytes());
        result.extend_from_slice(&self.txid.0);
        result.extend_from_slice(&self.public_key.0);
        Ok(result)
    }

    fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        // 8 + 8 + 8 + 32 + 33 = 89 bytes
        if bytes.len() != 89 {
            anyhow::bail!(
                "expected 49 bytes for deserializing L1Deposit, got {} bytes",
                bytes.len()
            );
        }
        let checkpoint_id = u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        let deposit_id = u64::from_be_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]);
        let value = u64::from_be_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23],
        ]);
        let mut txid = [0u8; 32];
        txid.copy_from_slice(&bytes[24..56]);

        let mut public_key = [0u8; 33];
        public_key.copy_from_slice(&bytes[56..89]);
        Ok(Self {
            deposit_id,
            checkpoint_id,
            value,
            txid: Hash256(txid),
            public_key: CompressedPublicKey(public_key),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, Hash, Eq, PartialEq)]
pub struct CityL1Withdrawal {
    pub withdrawal_id: u64,
    pub address: Hash160,
    pub address_type: u8,
    pub value: u64,
}

impl CityL1Withdrawal {
    pub fn from_hash<F: RichField>(id: u64, hash: QHashOut<F>) -> Self {
        /*

        let script_length = self.script.len();
        assert!(
            script_length == 23 || script_length == 25,
            "only supports p2sh (length = 23) and p2pkh withdrawals (length = 25), got length = {}",
            script_length
        );
        let public_key_hash_bytes = self.script[2..22]
            .iter()
            .map(|f| f.to_canonical_u64() as u8)
            .collect::<Vec<u8>>();
        let withdrawal_type_flag = if script_length == 23 {
            WITHDRAWAL_TYPE_P2SH
        } else {
            WITHDRAWAL_TYPE_P2PKH
        };
        let last_48_bits_with_flag =
            read_u48_from_bytes_le(&public_key_hash_bytes, 14) | withdrawal_type_flag;

        QHashOut(HashOut {
            elements: [
                self.value,
                F::from_noncanonical_u64(read_u56_from_bytes_le(&public_key_hash_bytes, 0)),
                F::from_noncanonical_u64(read_u56_from_bytes_le(&public_key_hash_bytes, 7)),
                F::from_noncanonical_u64(last_48_bits_with_flag),
            ],
        })
        */
        let value = hash.0.elements[0].to_canonical_u64();
        let mut address = [0u8; 20];
        let a = hash.0.elements[1].to_canonical_u64() & 0xffffffffffffffu64;
        let b = hash.0.elements[2].to_canonical_u64() & 0xffffffffffffffu64;
        let c = hash.0.elements[3].to_canonical_u64() & 0xffffffffffffffu64;
        address[0..7].copy_from_slice(&a.to_le_bytes()[0..7]);
        address[7..14].copy_from_slice(&b.to_le_bytes()[0..7]);
        address[14..20].copy_from_slice(&c.to_le_bytes()[0..6]);

        let address_type = ((c >> 48) & 0xff) as u8;

        Self {
            withdrawal_id: id,
            address: Hash160(address),
            address_type,
            value,
        }
    }
}