use city_common::{
    config::rollup_constants::{BLOCK_SCRIPT_SPEND_BASE_FEE_AMOUNT, WITHDRAWAL_FEE_AMOUNT},
    logging::debug_timer::DebugTimer,
};
use city_crypto::hash::base_types::{felt252::felt252_hashout_to_hash256_le, hash160::Hash160};
use city_rollup_common::{
    actors::traits::{OrchestratorEventReceiverSync, QBitcoinAPISync},
    api::data::{block::requested_actions::CityProcessWithdrawalRequest, store::CityL1Withdrawal},
    config::sighash_wrapper_config::SIGHASH_CIRCUIT_MAX_WITHDRAWALS,
    introspection::{
        rollup::introspection::BlockSpendIntrospectionHint,
        sighash::{SigHashPreimage, SIGHASH_ALL},
        transaction::{BTCTransaction, BTCTransactionInput, BTCTransactionOutput},
    },
    qworker::{fingerprints::CRWorkerToolboxCoreCircuitFingerprints, proof_store::QProofStore},
};
use city_store::{
    config::F,
    store::{city::base::CityStore, sighash::SigHashMerkleTree},
};
use kvq::traits::KVQBinaryStore;
use plonky2::{field::goldilocks_field::GoldilocksField, hash::hash_types::RichField};

use crate::debug::scenario::{
    block_planner::planner::CityOrchestratorBlockPlanner,
    requested_actions::CityScenarioRequestedActions,
    rpc_processor::CityScenarioRequestedActionsFromRPC, sighash::finalizer::SigHashFinalizer,
};

pub struct SimpleActorOrchestrator {
    pub fingerprints: CRWorkerToolboxCoreCircuitFingerprints<F>,
}
pub fn create_hints_for_block(
    current_script: &[u8],
    next_address: Hash160,
    next_script: &[u8],
    block_utxo: BTCTransaction,
    deposits: &[BTCTransaction],
    withdrawals: &[CityL1Withdrawal],
) -> anyhow::Result<Vec<BlockSpendIntrospectionHint>> {
    let base_inputs = [
        vec![block_utxo],
        deposits.to_vec().into_iter().skip(1).collect::<Vec<_>>(),
    ]
    .concat();

    let total_balance = base_inputs.iter().map(|x| x.outputs[0].value).sum::<u64>();
    let total_withdrawals = withdrawals.iter().map(|x| x.value).sum::<u64>();
    let total_fees =
        WITHDRAWAL_FEE_AMOUNT * (withdrawals.len() as u64) + BLOCK_SCRIPT_SPEND_BASE_FEE_AMOUNT;
    if total_fees > total_balance {
        anyhow::bail!("total fees exceed total balance");
    }

    let next_block_balance = total_balance - total_withdrawals;

    let outputs = [
        vec![BTCTransactionOutput {
            script: [vec![0xa9u8, 0x14u8], next_address.0.to_vec(), vec![0x87u8]].concat(),
            value: next_block_balance,
        }],
        withdrawals
            .iter()
            .map(|x| x.to_btc_tx_out())
            .collect::<Vec<_>>(),
    ]
    .concat();

    let base_tx = BTCTransaction {
        version: 2,
        inputs: base_inputs
            .iter()
            .map(|x| BTCTransactionInput {
                hash: x.get_hash(),
                sequence: 0xffffffff,
                script: vec![],
                index: 0,
            })
            .collect(),
        outputs,
        locktime: 0,
    };
    let base_sighash_preimage = SigHashPreimage {
        transaction: base_tx,
        sighash_type: SIGHASH_ALL,
    };

    let mut next_block_sighash_preimage_output = base_sighash_preimage.clone();
    next_block_sighash_preimage_output.transaction.inputs[0].script = current_script.to_vec();
    let hint = BlockSpendIntrospectionHint {
        sighash_preimage: next_block_sighash_preimage_output,
        last_block_spend_index: 0,
        block_spend_index: 0,
        current_spend_index: 0,
        funding_transactions: deposits.to_vec(),
        next_block_redeem_script: next_script.to_vec(),
    };
    let mut spend_hints: Vec<BlockSpendIntrospectionHint> = vec![hint];
    let inputs_len = base_inputs.len();
    for i in 0..inputs_len {
        let mut next_block_sighash_preimage_output = base_sighash_preimage.clone();
        next_block_sighash_preimage_output.transaction.inputs[i + 1].script =
            current_script.to_vec();
        let hint = BlockSpendIntrospectionHint {
            sighash_preimage: next_block_sighash_preimage_output,
            last_block_spend_index: 0,
            block_spend_index: 0,
            current_spend_index: i,
            funding_transactions: deposits.to_vec(),
            next_block_redeem_script: next_script.to_vec(),
        };
        spend_hints.push(hint);
    }

    Ok(spend_hints)
}
impl SimpleActorOrchestrator {
    pub fn produce_block<
        PS: QProofStore,
        S: KVQBinaryStore,
        BTC: QBitcoinAPISync,
        ER: OrchestratorEventReceiverSync<F>,
    >(
        proof_store: &mut PS,
        store: &mut S,
        event_receiver: &mut ER,
        btc_api: &mut BTC,
        fingerprints: &CRWorkerToolboxCoreCircuitFingerprints<F>,
        sighash_whitelist_tree: &SigHashMerkleTree,
    ) -> anyhow::Result<()> {
        let mut timer = DebugTimer::new("produce_block");
        let last_block = CityStore::get_latest_block_state(store)?;
        let last_block_address =
            CityStore::get_city_block_deposit_address(store, last_block.checkpoint_id)?;
        let last_block_script = CityStore::get_city_block_script(store, last_block.checkpoint_id)?;

        let checkpoint_id = last_block.checkpoint_id + 1;

        let register_users = event_receiver.flush_register_users()?;
        let claim_l1_deposits = event_receiver.flush_claim_deposits()?;
        let add_withdrawals = event_receiver.flush_add_withdrawals()?;
        let token_transfers = event_receiver.flush_token_transfers()?;

        let utxos = btc_api.get_utxos(last_block_address)?;
        let mut deposit_utxos = vec![BTCTransaction::dummy()];
        let mut last_block_utxo = BTCTransaction::dummy();
        for utxo in utxos.into_iter() {
            if utxo.is_p2pkh() {
                deposit_utxos.push(utxo);
            } else if utxo.is_block_spend_for_state(last_block_address) {
                last_block_utxo = utxo;
            }
        }
        if last_block_utxo.is_dummy() {
            anyhow::bail!("utxo not funded by last block");
        }

        let block_requested = CityScenarioRequestedActions::new_from_requested_rpc(
            CityScenarioRequestedActionsFromRPC {
                register_users,
                claim_l1_deposits,
                add_withdrawals,
                token_transfers,
            },
            &deposit_utxos,
            &last_block,
            SIGHASH_CIRCUIT_MAX_WITHDRAWALS,
        );

        let mut block_planner =
            CityOrchestratorBlockPlanner::<S, PS>::new(fingerprints.clone(), last_block);
        timer.lap("end process state block 1 RPC");
        timer.lap("start process requests block 1");

        let (block_op_job_ids, _block_state_transition, block_end_jobs, withdrawals) =
            block_planner.process_requests(store, proof_store, &block_requested)?;
        let final_state_root =
            felt252_hashout_to_hash256_le(CityStore::<S>::get_city_root(&store, 1)?.0);
        let next_address = CityStore::get_city_block_deposit_address(store, checkpoint_id)?;
        let next_script = CityStore::get_city_block_script(store, checkpoint_id)?;
        let hints = create_hints_for_block(
            &last_block_script,
            next_address,
            &next_script,
            last_block_utxo,
            &deposit_utxos,
            &withdrawals,
        )?;

        let sighash_jobs = SigHashFinalizer::finalize_sighashes::<PS>(
            proof_store,
            sighash_whitelist_tree,
            checkpoint_id,
            *block_end_jobs.last().unwrap(),
            &hints,
        )?;
        Ok(())
    }
}