use std::time::Duration;

use city_common::{cli::args::L2WorkerArgs, logging::trace_timer::TraceTimer};
use city_common_circuit::circuits::traits::qstandard::QStandardCircuitProvableWithProofStoreSync;
use city_common_circuit::circuits::traits::qstandard::QStandardCircuitWithDefaultMinified;
use city_macros::async_infinite_loop;
use city_rollup_circuit::block_circuits::ops::{
    add_l1_withdrawal::WCRAddL1WithdrawalCircuit,
    claim_l1_deposit::WCRClaimL1DepositCircuit,
    register_user::{CRUserRegistrationCircuitInput, WCRUserRegistrationCircuit},
};
use city_rollup_common::{
    introspection::rollup::constants::get_network_magic_for_str, qworker::job_id::QJobTopic,
};
use city_rollup_worker_dispatch::{
    implementations::redis::{RedisStore, Q_JOB},
    traits::proving_worker::ProvingWorkerListener,
};
use city_store::config::{C, D, F};
use tokio::task::spawn_blocking;

use crate::proof_store::SyncRedisProofStore;

// CRL2TransferCircuitInput
// CRUserRegistrationCircuitInput
// CRClaimL1DepositCircuitInput
// CRProcessL1WithdrawalCircuitInput
pub async fn run(args: L2WorkerArgs) -> anyhow::Result<()> {
    let redis_store = RedisStore::new(&args.redis_uri).await?;
    let proof_store = SyncRedisProofStore::new(&args.redis_uri)?;
    let network_magic = get_network_magic_for_str(args.network.to_string())?;

    let mut trace_timer = TraceTimer::new("CRWorkerToolboxCoreCircuits");
    trace_timer.lap("start => build core toolbox circuits");
    let op_register_user =
        WCRUserRegistrationCircuit::<C, D>::new_default_with_minifiers(network_magic, 1);

    trace_timer.lap("built op_register_user");
    async_infinite_loop!(1000, {
        let proof_store = proof_store.clone();
        let mut redis_store = redis_store.clone();
        while let Ok(message) = redis_store
            .get_next_message::<Q_JOB>(QJobTopic::GenerateStandardProof as u32)
            .await
        {
            if let Ok(register_user) =
                serde_json::from_slice::<CRUserRegistrationCircuitInput<F>>(&message)
            {
                // TODO: spawn blocking
                op_register_user.prove_with_proof_store_sync(&proof_store, &register_user)?;
            }
        }
    });
}
