use city_crypto::hash::base_types::hash256::Hash256;
use city_rollup_common::api::data::store::CityL1Deposit;
use kvq::traits::KVQSerializable;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct L1DepositKeyByTransactionIdCore<const TABLE_TYPE: u16>(pub [u8; 32]);

impl<const TABLE_TYPE: u16> KVQSerializable for L1DepositKeyByTransactionIdCore<TABLE_TYPE> {
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(vec![
            (TABLE_TYPE >> 8) as u8,
            (TABLE_TYPE & 0xff) as u8,
            self.0[0],
            self.0[1],
            self.0[2],
            self.0[3],
            self.0[4],
            self.0[5],
            self.0[6],
            self.0[7],
            self.0[8],
            self.0[9],
            self.0[10],
            self.0[11],
            self.0[12],
            self.0[13],
            self.0[14],
            self.0[15],
            self.0[16],
            self.0[17],
            self.0[18],
            self.0[19],
            self.0[20],
            self.0[21],
            self.0[22],
            self.0[23],
            self.0[24],
            self.0[25],
            self.0[26],
            self.0[27],
            self.0[28],
            self.0[29],
            self.0[30],
            self.0[31],
        ])
    }

    fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.len() != 34 {
            anyhow::bail!(
                "expected 34 bytes for deserializing L1DepositKeyByTransactionIdCore, got {} bytes",
                bytes.len()
            );
        }
        let mut inner_data = [0u8; 32];
        inner_data.copy_from_slice(&bytes[2..]);
        Ok(L1DepositKeyByTransactionIdCore(inner_data))
    }
}
impl<const TABLE_TYPE: u16> From<&CityL1Deposit> for L1DepositKeyByTransactionIdCore<TABLE_TYPE> {
    fn from(deposit: &CityL1Deposit) -> Self {
        L1DepositKeyByTransactionIdCore(deposit.txid.0)
    }
}
impl<const TABLE_TYPE: u16> From<&Hash256> for L1DepositKeyByTransactionIdCore<TABLE_TYPE> {
    fn from(txid: &Hash256) -> Self {
        L1DepositKeyByTransactionIdCore(txid.0)
    }
}

impl<const TABLE_TYPE: u16> From<[u8; 32]> for L1DepositKeyByTransactionIdCore<TABLE_TYPE> {
    fn from(txid: [u8; 32]) -> Self {
        L1DepositKeyByTransactionIdCore(txid)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct L1DepositKeyByDepositIdCore<const TABLE_TYPE: u16> {
    pub deposit_id: u64,
    pub checkpoint_id: u64,
}

impl<const TABLE_TYPE: u16> KVQSerializable for L1DepositKeyByDepositIdCore<TABLE_TYPE> {
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let deposit_id_be_bytes = self.deposit_id.to_be_bytes();
        let checkpoint_id_be_bytes = self.checkpoint_id.to_be_bytes();
        Ok(vec![
            (TABLE_TYPE >> 8) as u8,
            (TABLE_TYPE & 0xff) as u8,
            deposit_id_be_bytes[0],
            deposit_id_be_bytes[1],
            deposit_id_be_bytes[2],
            deposit_id_be_bytes[3],
            deposit_id_be_bytes[4],
            deposit_id_be_bytes[5],
            deposit_id_be_bytes[6],
            deposit_id_be_bytes[7],
            checkpoint_id_be_bytes[0],
            checkpoint_id_be_bytes[1],
            checkpoint_id_be_bytes[2],
            checkpoint_id_be_bytes[3],
            checkpoint_id_be_bytes[4],
            checkpoint_id_be_bytes[5],
            checkpoint_id_be_bytes[6],
            checkpoint_id_be_bytes[7],
        ])
    }

    fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.len() != 18 {
            anyhow::bail!(
                "expected 18 bytes for deserializing L1DepositKeyByDepositIdCore, got {} bytes",
                bytes.len()
            );
        }
        let mut deposit_id_be_bytes = [0u8; 8];
        deposit_id_be_bytes.copy_from_slice(&bytes[2..10]);
        let deposit_id = u64::from_be_bytes(deposit_id_be_bytes);

        let mut checkpoint_id_be_bytes = [0u8; 8];
        checkpoint_id_be_bytes.copy_from_slice(&bytes[10..18]);
        let checkpoint_id = u64::from_be_bytes(checkpoint_id_be_bytes);

        Ok(L1DepositKeyByDepositIdCore {
            deposit_id,
            checkpoint_id,
        })
    }
}
impl<const TABLE_TYPE: u16> From<&CityL1Deposit> for L1DepositKeyByDepositIdCore<TABLE_TYPE> {
    fn from(deposit: &CityL1Deposit) -> Self {
        L1DepositKeyByDepositIdCore::new(deposit.checkpoint_id, deposit.deposit_id)
    }
}
impl<const TABLE_TYPE: u16> L1DepositKeyByDepositIdCore<TABLE_TYPE> {
    pub fn new(checkpoint_id: u64, deposit_id: u64) -> Self {
        L1DepositKeyByDepositIdCore {
            deposit_id,
            checkpoint_id,
        }
    }
}
