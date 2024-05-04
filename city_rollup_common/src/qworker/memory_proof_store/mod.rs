use std::collections::HashMap;

use plonky2::plonk::{config::GenericConfig, proof::ProofWithPublicInputs};

use super::{
    job_id::QProvingJobDataID,
    proof_store::{QProofStoreReaderSync, QProofStoreWriterSync},
};

pub struct SimpleProofStoreMemory {
    pub proofs: HashMap<QProvingJobDataID, Vec<u8>>,
    pub counters: HashMap<QProvingJobDataID, u32>,
}
impl SimpleProofStoreMemory {
    pub fn new() -> Self {
        Self {
            proofs: HashMap::new(),
            counters: HashMap::new(),
        }
    }
}

impl QProofStoreReaderSync for SimpleProofStoreMemory {
    fn get_proof_by_id<C: GenericConfig<D>, const D: usize>(
        &self,
        id: QProvingJobDataID,
    ) -> anyhow::Result<ProofWithPublicInputs<C::F, C, D>> {
        let data = self
            .proofs
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("Proof not found"))?;
        Ok(bincode::deserialize(data)?)
    }
}

impl QProofStoreWriterSync for SimpleProofStoreMemory {
    fn set_proof_by_id<C: GenericConfig<D>, const D: usize>(
        &mut self,
        id: QProvingJobDataID,
        proof: &ProofWithPublicInputs<C::F, C, D>,
    ) -> anyhow::Result<()> {
        self.proofs.insert(id, bincode::serialize(proof)?);
        Ok(())
    }

    fn inc_counter_by_id<C: GenericConfig<D>, const D: usize>(
        &mut self,
        id: QProvingJobDataID,
    ) -> anyhow::Result<u32> {
        let zero = 0u32;
        let new_value = 1 + *(self.counters.get(&id).unwrap_or(&zero));
        self.counters.insert(id, new_value);
        Ok(new_value)
    }
}