use city_crypto::hash::base_types::hash256::Hash256;
use serde::{Deserialize, Serialize};

use serde_with::serde_as;

#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct CityRollupDeposit {
    pub sighash: Hash256,

    #[serde_as(as = "serde_with::hex::Hex")]
    pub sighash_preimage: Vec<u8>,

    pub index: usize,
    pub txid: Hash256,

    #[serde_as(as = "serde_with::hex::Hex")]
    pub funding_tx: Vec<u8>,
}
