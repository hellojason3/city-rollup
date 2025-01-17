use std::marker::PhantomData;

use city_common::config::rollup_constants::{DEPOSIT_FEE_AMOUNT, WITHDRAWAL_FEE_AMOUNT};
use city_crypto::hash::qhashout::QHashOut;
use city_rollup_common::{
    api::data::{
        block::requested_actions::{
            CityAddDepositRequest, CityAddWithdrawalRequest, CityClaimDepositRequest,
            CityProcessWithdrawalRequest, CityRegisterUserRequest, CityTokenTransferRequest,
        },
        store::CityL2BlockState,
    },
    introspection::rollup::introspection_result::BTCRollupIntrospectionResultDeposit,
    qworker::{
        fingerprints::CRWorkerToolboxCoreCircuitFingerprints,
        job_witnesses::op::{
            CRAddL1DepositCircuitInput, CRAddL1WithdrawalCircuitInput,
            CRClaimL1DepositCircuitInput, CRL2TransferCircuitInput,
            CRProcessL1WithdrawalCircuitInput, CRUserRegistrationCircuitInput,
        },
    },
};
use city_store::{config::F, store::city::base::CityStore};
use kvq::traits::KVQBinaryStore;

pub struct CityOrchestratorOpRequestProcessor<S: KVQBinaryStore> {
    pub last_block_state: CityL2BlockState,
    pub checkpoint_id: u64,
    pub next_add_withdrawal_id: u64,
    pub next_process_withdrawal_id: u64,
    pub next_deposit_id: u64,
    pub next_user_id: u64,
    pub total_deposits_claimed_epoch: u64,
    pub fingerprints: CRWorkerToolboxCoreCircuitFingerprints<F>,
    pub processed_withdrawal_hashes: Vec<QHashOut<F>>,
    pub added_deposit_hashes: Vec<QHashOut<F>>,
    pub block_total_deposited: u64,
    pub block_total_withdrawn: u64,

    _store: PhantomData<S>,
}

impl<S: KVQBinaryStore> CityOrchestratorOpRequestProcessor<S> {
    pub fn new(
        last_block_state: CityL2BlockState,
        fingerprints: CRWorkerToolboxCoreCircuitFingerprints<F>,
    ) -> Self {
        Self {
            last_block_state,
            checkpoint_id: last_block_state.checkpoint_id + 1,
            next_add_withdrawal_id: if last_block_state.next_add_withdrawal_id == last_block_state.next_process_withdrawal_id{
                0
            }else{
                last_block_state.next_add_withdrawal_id
            },
            next_process_withdrawal_id: if last_block_state.next_add_withdrawal_id == last_block_state.next_process_withdrawal_id{
                0
            }else{
                last_block_state.next_process_withdrawal_id
            },
            next_deposit_id: last_block_state.next_deposit_id,
            next_user_id: last_block_state.next_user_id,
            total_deposits_claimed_epoch: last_block_state.total_deposits_claimed_epoch,
            fingerprints,
            added_deposit_hashes: Vec::new(),
            processed_withdrawal_hashes: Vec::new(),
            block_total_deposited: 0,
            block_total_withdrawn: 0,

            _store: PhantomData,
        }
    }

    pub fn get_finalized_block_state(&self) -> CityL2BlockState {
        CityL2BlockState {
            checkpoint_id: self.checkpoint_id,
            next_add_withdrawal_id: if self.next_add_withdrawal_id == self.next_process_withdrawal_id {
                0
            }else{
                self.next_add_withdrawal_id
            },
            next_process_withdrawal_id: if self.next_add_withdrawal_id == self.next_process_withdrawal_id {
                0
            }else{
                self.next_process_withdrawal_id
            },
            next_deposit_id: self.next_deposit_id,
            next_user_id: self.next_user_id,
            total_deposits_claimed_epoch: self.total_deposits_claimed_epoch,
            end_balance: self.last_block_state.end_balance + self.block_total_deposited
                - self.block_total_withdrawn,
        }
    }

    pub fn process_add_deposit_request(
        &mut self,
        store: &mut S,
        req: &CityAddDepositRequest,
    ) -> anyhow::Result<CRAddL1DepositCircuitInput<F>> {
        assert!(
            req.value > DEPOSIT_FEE_AMOUNT,
            "deposits must be larger than the deposit_fee amount"
        );
        let deposit_id = self.next_deposit_id;
        let deposit_tree_delta_merkle_proof =
            CityStore::<S>::add_deposit_from_request(store, self.checkpoint_id, deposit_id, req)?;
        self.added_deposit_hashes
            .push(deposit_tree_delta_merkle_proof.new_value);
        self.next_deposit_id += 1;
        self.block_total_deposited += req.value - DEPOSIT_FEE_AMOUNT;
        Ok(CRAddL1DepositCircuitInput {
            deposit_tree_delta_merkle_proof,
            allowed_circuit_hashes_root: self
                .fingerprints
                .op_add_l1_deposit
                .allowed_circuit_hashes_root,
        })
    }

    pub fn process_add_withdrawal_request(
        &mut self,
        store: &mut S,
        req: &CityAddWithdrawalRequest,
    ) -> anyhow::Result<CRAddL1WithdrawalCircuitInput<F>> {
        let withdrawal_id = self.next_add_withdrawal_id;
        let user_tree_delta_merkle_proof = CityStore::<S>::decrement_user_balance(
            store,
            self.checkpoint_id,
            req.user_id,
            req.value + WITHDRAWAL_FEE_AMOUNT,
            Some(req.nonce),
        )?;
        self.block_total_withdrawn += req.value + WITHDRAWAL_FEE_AMOUNT;
        let withdrawal_tree_delta_merkle_proof =
            CityStore::<S>::add_withdrawal_to_tree_from_request(
                store,
                self.checkpoint_id,
                withdrawal_id,
                req,
            )?;
        self.next_add_withdrawal_id += 1;
        Ok(CRAddL1WithdrawalCircuitInput {
            allowed_circuit_hashes_root: self
                .fingerprints
                .op_add_l1_withdrawal
                .allowed_circuit_hashes_root,
            user_tree_delta_merkle_proof,
            withdrawal_tree_delta_merkle_proof,
            signature_proof_id: req.signature_proof_id,
        })
    }
    pub fn process_claim_deposit_request(
        &mut self,
        store: &mut S,
        req: &CityClaimDepositRequest,
    ) -> anyhow::Result<CRClaimL1DepositCircuitInput<F>> {
        let deposit_tree_delta_merkle_proof =
            CityStore::<S>::mark_deposit_as_claimed(store, self.checkpoint_id, req.deposit_id)?;
        assert!(
            req.value > DEPOSIT_FEE_AMOUNT,
            "deposits must be larger than the deposit_fee amount"
        );
        let user_tree_delta_merkle_proof = CityStore::<S>::increment_user_balance(
            store,
            self.checkpoint_id,
            req.user_id,
            req.value - DEPOSIT_FEE_AMOUNT,
            None,
        )?;
        let deposit = BTCRollupIntrospectionResultDeposit::from_byte_representation(
            &req.public_key.0,
            req.txid,
            req.value,
        );

        self.total_deposits_claimed_epoch += 1;
        Ok(CRClaimL1DepositCircuitInput {
            deposit_tree_delta_merkle_proof,
            allowed_circuit_hashes_root: self
                .fingerprints
                .op_claim_l1_deposit
                .allowed_circuit_hashes_root,
            deposit,
            user_tree_delta_merkle_proof,
            signature_proof_id: req.signature_proof_id,
        })
    }

    pub fn process_l2_transfer_request(
        &mut self,
        store: &mut S,
        req: &CityTokenTransferRequest,
    ) -> anyhow::Result<CRL2TransferCircuitInput<F>> {
        let sender_user_tree_delta_merkle_proof = CityStore::<S>::decrement_user_balance(
            store,
            self.checkpoint_id,
            req.user_id,
            req.value,
            Some(req.nonce),
        )?;

        let receiver_user_tree_delta_merkle_proof = CityStore::<S>::increment_user_balance(
            store,
            self.checkpoint_id,
            req.to,
            req.value,
            None,
        )?;

        Ok(CRL2TransferCircuitInput {
            sender_user_tree_delta_merkle_proof,
            receiver_user_tree_delta_merkle_proof,
            allowed_circuit_hashes_root: self
                .fingerprints
                .op_l2_transfer
                .allowed_circuit_hashes_root,
            signature_proof_id: req.signature_proof_id,
        })
    }
    pub fn process_complete_l1_withdrawal_request(
        &mut self,
        store: &mut S,
        req: &CityProcessWithdrawalRequest,
    ) -> anyhow::Result<CRProcessL1WithdrawalCircuitInput<F>> {
        let withdrawal_tree_delta_merkle_proof = CityStore::<S>::mark_withdrawal_as_completed(
            store,
            self.checkpoint_id,
            req.withdrawal_id,
        )?;
        self.processed_withdrawal_hashes
            .push(withdrawal_tree_delta_merkle_proof.new_value);
        self.next_process_withdrawal_id += 1;
        Ok(CRProcessL1WithdrawalCircuitInput {
            withdrawal_tree_delta_merkle_proof,
            allowed_circuit_hashes_root: self
                .fingerprints
                .op_process_l1_withdrawal
                .allowed_circuit_hashes_root,
        })
    }
    pub fn process_register_user_request(
        &mut self,
        store: &mut S,
        req: &CityRegisterUserRequest<F>,
    ) -> anyhow::Result<CRUserRegistrationCircuitInput<F>> {
        let user_tree_delta_merkle_proof = CityStore::<S>::register_user(
            store,
            self.checkpoint_id,
            self.next_user_id,
            req.public_key,
        )?;
        self.next_user_id += 1;
        Ok(CRUserRegistrationCircuitInput {
            user_tree_delta_merkle_proof,
            allowed_circuit_hashes_root: self
                .fingerprints
                .op_register_user
                .allowed_circuit_hashes_root,
        })
    }
}
