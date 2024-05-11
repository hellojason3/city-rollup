use city_common::binaryhelpers::bytes::CompressedPublicKey;
use city_crypto::hash::base_types::hash160::Hash160;
use city_crypto::hash::base_types::hash256::Hash256;
use city_crypto::hash::qhashout::QHashOut;
use plonky2::hash::hash_types::RichField;
use serde::Deserialize;
use serde::Serialize;

use crate::introspection::transaction::BTCTransaction;
use crate::qworker::job_id::QProvingJobDataID;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Copy, PartialEq, Eq)]
pub struct CityTokenTransferRequest {
    request_type: u8,
    pub user_id: u64,
    pub to: u64,
    pub value: u64,
    pub nonce: u64,
    pub signature_proof_id: QProvingJobDataID,
}
impl CityTokenTransferRequest {
    pub fn new(
        user_id: u64,
        to: u64,
        value: u64,
        nonce: u64,
        signature_proof_id: QProvingJobDataID,
    ) -> Self {
        Self {
            request_type: 0,
            user_id,
            to,
            value,
            nonce,
            signature_proof_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityClaimDepositRequest {
    request_type: u8,
    pub user_id: u64,
    pub nonce: u64,
    pub deposit_id: u64,
    pub value: u64,

    pub txid: Hash256,
    pub public_key: CompressedPublicKey,
    pub signature_proof_id: QProvingJobDataID,
}
impl CityClaimDepositRequest {
    pub fn new(
        user_id: u64,
        nonce: u64,
        deposit_id: u64,
        value: u64,
        txid: Hash256,
        public_key: [u8; 33],
        signature_proof_id: QProvingJobDataID,
    ) -> Self {
        Self {
            request_type: 1,
            user_id,
            nonce,
            deposit_id,
            value,
            txid,
            public_key: CompressedPublicKey(public_key),
            signature_proof_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityAddDepositRequest {
    request_type: u8,
    pub value: u64,
    pub txid: Hash256,
    pub public_key: CompressedPublicKey,
}
impl CityAddDepositRequest {
    pub fn new(value: u64, txid: Hash256, public_key: [u8; 33]) -> Self {
        Self {
            request_type: 2,
            value,
            txid,
            public_key: CompressedPublicKey(public_key),
        }
    }
    pub fn new_from_transaction(funding_tx: &BTCTransaction) -> Self {
        assert_eq!(
            funding_tx.inputs.len(),
            1,
            "deposits should only have one input (p2pkh)"
        );
        assert_eq!(
            funding_tx.outputs.len(),
            1,
            "deposits should only have one output (send to layer 2)"
        );
        assert_eq!(
            funding_tx.inputs[0].script.len(),
            106,
            "the input script for a deposit should be a p2pkh signature + public key reveal"
        );

        let public_key = CompressedPublicKey::new_from_slice(&funding_tx.inputs[0].script[73..106]);
        Self {
            request_type: 2,
            value: funding_tx.outputs[0].value,
            txid: funding_tx.get_hash(),
            public_key: public_key,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityAddWithdrawalRequest {
    request_type: u8,
    pub user_id: u64,
    pub value: u64,
    pub nonce: u64,

    pub withdrawal_id: u64,

    pub destination_type: u8,
    pub destination: Hash160,
    pub signature_proof_id: QProvingJobDataID,
}

impl CityAddWithdrawalRequest {
    pub fn new(
        user_id: u64,
        value: u64,
        nonce: u64,
        withdrawal_id: u64,
        destination_type: u8,
        destination: Hash160,
        signature_proof_id: QProvingJobDataID,
    ) -> Self {
        Self {
            request_type: 3,
            user_id,
            value,
            nonce,
            withdrawal_id,
            destination_type,
            destination,
            signature_proof_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityProcessWithdrawalRequest {
    request_type: u8,
    pub withdrawal_id: u64,
}

impl CityProcessWithdrawalRequest {
    pub fn new(withdrawal_id: u64) -> Self {
        Self {
            request_type: 4,
            withdrawal_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct CityRegisterUserRequest<F: RichField> {
    request_type: u8,
    pub public_key: QHashOut<F>,
}

impl<F: RichField> CityRegisterUserRequest<F> {
    pub fn new(public_key: QHashOut<F>) -> Self {
        Self {
            request_type: 5,
            public_key,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
#[serde(untagged)]
pub enum CityRequest<F: RichField> {
    CityTokenTransferRequest((u32, CityTokenTransferRequest)),
    CityClaimDepositRequest((u32, CityClaimDepositRequest)),
    CityAddWithdrawalRequest((u32, CityAddWithdrawalRequest)),
    CityRegisterUserRequest((u32, CityRegisterUserRequest<F>)),
    CityAddDepositRequest((u32, CityAddDepositRequest)),
    CityProcessWithdrawalRequest((u32, CityProcessWithdrawalRequest)),
}
