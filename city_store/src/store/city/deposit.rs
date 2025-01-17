use city_crypto::hash::base_types::hash256::Hash256;
use city_rollup_common::{
    api::data::{block::requested_actions::CityAddDepositRequest, store::CityL1Deposit},
    introspection::rollup::introspection_result::BTCRollupIntrospectionResultDeposit,
};
use kvq::traits::{KVQBinaryStore, KVQBinaryStoreReader};

use crate::{
    config::{
        CityDeltaMerkleProof, CityHash, CityHasher, CityMerkleProof, L1DepositTreeStore,
        L1DepositsStore, F,
    },
    models::{
        kvq_merkle::model::{
            KVQFixedConfigMerkleTreeModelCore, KVQFixedConfigMerkleTreeModelReaderCore,
        },
        l1_deposits::model::{L1DepositsModelCore, L1DepositsModelReaderCore},
    },
};

use super::base::CityStore;

impl<S: KVQBinaryStoreReader> CityStore<S> {
    pub fn get_deposit_tree_root(store: &S, checkpoint_id: u64) -> anyhow::Result<CityHash> {
        L1DepositTreeStore::get_root_fc(store, checkpoint_id)
    }
    pub fn get_deposit_by_id(
        store: &S,
        checkpoint_id: u64,
        deposit_id: u64,
    ) -> anyhow::Result<CityL1Deposit> {
        L1DepositsStore::get_deposit_by_id(store, checkpoint_id, deposit_id)
    }
    pub fn get_deposits_by_id(
        store: &S,
        checkpoint_id: u64,
        deposit_ids: &[u64],
    ) -> anyhow::Result<Vec<CityL1Deposit>> {
        L1DepositsStore::get_deposits_by_id(store, checkpoint_id, deposit_ids)
    }
    pub fn get_deposit_by_txid(
        store: &S,
        transaction_id: Hash256,
    ) -> anyhow::Result<CityL1Deposit> {
        L1DepositsStore::get_deposit_by_txid(store, transaction_id)
    }
    pub fn get_deposits_by_txid(
        store: &S,
        transaction_ids: &[Hash256],
    ) -> anyhow::Result<Vec<CityL1Deposit>> {
        L1DepositsStore::get_deposits_by_txid(store, transaction_ids)
    }
    pub fn get_deposit_hash(
        store: &S,
        checkpoint_id: u64,
        deposit_id: u64,
    ) -> anyhow::Result<CityHash> {
        L1DepositTreeStore::get_leaf_value_fc(store, checkpoint_id, deposit_id)
    }
    pub fn get_deposit_leaf_merkle_proof(
        store: &S,
        checkpoint_id: u64,
        deposit_id: u64,
    ) -> anyhow::Result<CityMerkleProof> {
        L1DepositTreeStore::get_leaf_fc(store, checkpoint_id, deposit_id)
    }
}

impl<S: KVQBinaryStore> CityStore<S> {
    pub fn set_deposit(
        store: &mut S,
        checkpoint_id: u64,
        deposit: &CityL1Deposit,
    ) -> anyhow::Result<CityDeltaMerkleProof> {
        let deposit_hash = BTCRollupIntrospectionResultDeposit::<F>::from_byte_representation(
            &deposit.public_key.0,
            deposit.txid,
            deposit.value,
        )
        .get_hash::<CityHasher>();

        L1DepositsStore::set_deposit_ref(store, deposit)?;
        L1DepositTreeStore::set_leaf_fc(store, checkpoint_id, deposit.deposit_id, deposit_hash)
    }
    pub fn add_deposit_from_request(
        store: &mut S,
        checkpoint_id: u64,
        deposit_id: u64,
        req: &CityAddDepositRequest,
    ) -> anyhow::Result<CityDeltaMerkleProof> {
        let deposit = CityL1Deposit {
            deposit_id,
            checkpoint_id,
            value: req.value,
            txid: req.txid,
            public_key: req.public_key,
        };
        let deposit_hash = BTCRollupIntrospectionResultDeposit::<F>::from_byte_representation(
            &deposit.public_key.0,
            deposit.txid,
            deposit.value,
        )
        .get_hash::<CityHasher>();

        L1DepositsStore::set_deposit(store, deposit)?;
        L1DepositTreeStore::set_leaf_fc(store, checkpoint_id, deposit.deposit_id, deposit_hash)
    }
    pub fn mark_deposit_as_claimed(
        store: &mut S,
        checkpoint_id: u64,
        deposit_id: u64,
    ) -> anyhow::Result<CityDeltaMerkleProof> {
        L1DepositTreeStore::set_leaf_fc(store, checkpoint_id, deposit_id, CityHash::ZERO)
    }
}
