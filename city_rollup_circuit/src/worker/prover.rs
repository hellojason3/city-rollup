use city_common::logging::trace_timer::TraceTimer;
use city_rollup_common::qworker::{
    job_id::QProvingJobDataID,
    proof_store::{QProofStore, QProofStoreReaderSync},
};
use plonky2::plonk::config::GenericConfig;

use super::traits::QWorkerGenericProver;

pub struct QWorkerStandardProver {
    pub timer: TraceTimer,
}

impl QWorkerStandardProver {
    pub fn new() -> Self {
        Self {
            timer: TraceTimer::new("worker"),
        }
    }
    pub fn prove<
        S: QProofStore,
        G: QWorkerGenericProver<S, C, D>,
        C: GenericConfig<D>,
        const D: usize,
    >(
        &mut self,
        store: &mut S,
        prover: &G,
        job_id: QProvingJobDataID,
    ) -> anyhow::Result<QProvingJobDataID> {
        let proof = prover.worker_prove(store, job_id)?;
        let output_id = job_id.get_output_id();
        store.set_proof_by_id(output_id, &proof)?;

        Ok(output_id)
    }
}