use plonky2::{
    field::extension::Extendable,
    gates::gate::GateRef,
    hash::hash_types::{HashOut, RichField},
    plonk::{
        circuit_data::{CircuitConfig, CircuitData, CommonCircuitData, VerifierOnlyCircuitData},
        config::{AlgebraicHasher, GenericConfig},
        proof::ProofWithPublicInputs,
    },
};

use super::{pm_core::QEDProofMinifier, pm_custom::PMCircuitCustomizer};

#[derive(Debug)]
pub struct QEDProofMinifierChain<
    const D: usize,
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
> where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    pub minifiers: Vec<QEDProofMinifier<D, F, C>>,
}

impl<const D: usize, F: RichField + Extendable<D>, C: GenericConfig<D, F = F> + 'static>
    QEDProofMinifierChain<D, F, C>
where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    pub fn new_with_cfg(
        base_circuit_verifier_data: &VerifierOnlyCircuitData<C, D>,
        base_circuit_common_data: &CommonCircuitData<F, D>,
        minifier_configs: Vec<CircuitConfig>,
    ) -> Self {
        let mut minifiers = vec![QEDProofMinifier::<D, F, C>::new_with_cfg(
            minifier_configs[0].clone(),
            base_circuit_verifier_data,
            base_circuit_common_data,
            None,
        )];
        for i in 1..minifier_configs.len() {
            minifiers.push(QEDProofMinifier::<D, F, C>::new_with_cfg(
                minifier_configs[i].clone(),
                &minifiers[i - 1].circuit_data.verifier_only,
                &minifiers[i - 1].circuit_data.common,
                None,
            ));
        }
        /*
        for m in &minifiers {
          tracing::info!("deg_bits: {} ",m.common_circuit_data.degree_bits());
        }
        */

        Self { minifiers }
    }

    pub fn new_with_cfg_customizer<PMCC: PMCircuitCustomizer<F, D>>(
        base_circuit_verifier_data: &VerifierOnlyCircuitData<C, D>,
        base_circuit_common_data: &CommonCircuitData<F, D>,
        n_minifiers: usize,
        customizer: &PMCC,
    ) -> Self {
        let mut minifiers = vec![if n_minifiers == 1 {
            QEDProofMinifier::<D, F, C>::new_with_cfg_customizer(
                CircuitConfig::standard_recursion_config(),
                base_circuit_verifier_data,
                base_circuit_common_data,
                None,
                Some(customizer),
            )
        } else {
            QEDProofMinifier::<D, F, C>::new_with_cfg(
                CircuitConfig::standard_recursion_config(),
                base_circuit_verifier_data,
                base_circuit_common_data,
                None,
            )
        }];
        for i in 1..n_minifiers {
            if i == (n_minifiers - 1) {
                minifiers.push(QEDProofMinifier::<D, F, C>::new_with_cfg_customizer(
                    CircuitConfig::standard_recursion_config(),
                    &minifiers[i - 1].circuit_data.verifier_only,
                    &minifiers[i - 1].circuit_data.common,
                    None,
                    Some(customizer),
                ));
            } else {
                minifiers.push(QEDProofMinifier::<D, F, C>::new_with_cfg(
                    CircuitConfig::standard_recursion_config(),
                    &minifiers[i - 1].circuit_data.verifier_only,
                    &minifiers[i - 1].circuit_data.common,
                    None,
                ));
            }
        }
        /*
        for m in &minifiers {
          tracing::info!("deg_bits: {} ",m.common_circuit_data.degree_bits());
        }
        */

        Self { minifiers }
    }

    pub fn new_with_cfg_customizer_add_gates<PMCC: PMCircuitCustomizer<F, D>>(
        base_circuit_verifier_data: &VerifierOnlyCircuitData<C, D>,
        base_circuit_common_data: &CommonCircuitData<F, D>,
        n_minifiers: usize,
        add_gates: Option<&[GateRef<F, D>]>,
        customizer: Option<&PMCC>,
    ) -> Self {
        let mut minifiers = vec![if n_minifiers == 1 {
            QEDProofMinifier::<D, F, C>::new_with_cfg_customizer(
                CircuitConfig::standard_recursion_config(),
                base_circuit_verifier_data,
                base_circuit_common_data,
                add_gates,
                customizer,
            )
        } else {
            QEDProofMinifier::<D, F, C>::new_with_cfg(
                CircuitConfig::standard_recursion_config(),
                base_circuit_verifier_data,
                base_circuit_common_data,
                None,
            )
        }];
        for i in 1..n_minifiers {
            if i == (n_minifiers - 1) {
                minifiers.push(QEDProofMinifier::<D, F, C>::new_with_cfg_customizer(
                    CircuitConfig::standard_recursion_config(),
                    &minifiers[i - 1].circuit_data.verifier_only,
                    &minifiers[i - 1].circuit_data.common,
                    add_gates,
                    customizer,
                ));
            } else {
                minifiers.push(QEDProofMinifier::<D, F, C>::new_with_cfg(
                    CircuitConfig::standard_recursion_config(),
                    &minifiers[i - 1].circuit_data.verifier_only,
                    &minifiers[i - 1].circuit_data.common,
                    None,
                ));
            }
        }
        /*
        for m in &minifiers {
          tracing::info!("deg_bits: {} ",m.common_circuit_data.degree_bits());
        }
        */

        Self { minifiers }
    }
    pub fn new(
        base_circuit_verifier_data: &VerifierOnlyCircuitData<C, D>,
        base_circuit_common_data: &CommonCircuitData<F, D>,
        n_minifiers: usize,
    ) -> Self {
        if n_minifiers == 0 {
            return Self { minifiers: vec![] };
        }
        let mut minifiers = vec![QEDProofMinifier::<D, F, C>::new(
            base_circuit_verifier_data,
            base_circuit_common_data,
        )];
        for i in 1..n_minifiers {
            minifiers.push(QEDProofMinifier::<D, F, C>::new(
                &minifiers[i - 1].circuit_data.verifier_only,
                &minifiers[i - 1].circuit_data.common,
            ));
        }
        /*
        for m in &minifiers {
            tracing::info!("deg_bits: {} ", m.circuit_data.common.degree_bits());
        }*/

        Self { minifiers }
    }
    pub fn new_add_gates(
        base_circuit_verifier_data: &VerifierOnlyCircuitData<C, D>,
        base_circuit_common_data: &CommonCircuitData<F, D>,
        n_minifiers: usize,
        add_gates: Option<&[GateRef<F, D>]>,
    ) -> Self {
        let mut minifiers = vec![if n_minifiers == 1 {
            QEDProofMinifier::<D, F, C>::new_with_cfg(
                CircuitConfig::standard_recursion_config(),
                base_circuit_verifier_data,
                base_circuit_common_data,
                add_gates,
            )
        } else {
            QEDProofMinifier::<D, F, C>::new_with_cfg(
                CircuitConfig::standard_recursion_config(),
                base_circuit_verifier_data,
                base_circuit_common_data,
                None,
            )
        }];
        for i in 1..n_minifiers {
            if i == (n_minifiers - 1) {
                minifiers.push(QEDProofMinifier::<D, F, C>::new_with_cfg(
                    CircuitConfig::standard_recursion_config(),
                    &minifiers[i - 1].circuit_data.verifier_only,
                    &minifiers[i - 1].circuit_data.common,
                    add_gates,
                ));
            } else {
                minifiers.push(QEDProofMinifier::<D, F, C>::new_with_cfg(
                    CircuitConfig::standard_recursion_config(),
                    &minifiers[i - 1].circuit_data.verifier_only,
                    &minifiers[i - 1].circuit_data.common,
                    None,
                ));
            }
        }
        /*
        for m in &minifiers {
          tracing::info!("deg_bits: {} ",m.common_circuit_data.degree_bits());
        }
        */

        Self { minifiers }
    }
    pub fn prove(
        &self,
        base_proof: &ProofWithPublicInputs<F, C, D>, //verifier_data: &VerifierOnlyCircuitData<C, D>,
                                                     //proof: &ProofWithPublicInputs<F, C, D>,
    ) -> anyhow::Result<ProofWithPublicInputs<F, C, D>> {
        if self.minifiers.len() == 0 {
            return Ok(base_proof.clone());
        }
        let mut tmp_proof = self.minifiers[0].prove(base_proof)?;
        let len = self.minifiers.len();
        let lenm1 = len - 1;
        for i in 1..len {
            tmp_proof = self.minifiers[i].prove(&tmp_proof)?;
            if i == lenm1 {
                return Ok(tmp_proof);
            }
        }
        Ok(tmp_proof)
    }

    pub fn get_fingerprint(&self) -> HashOut<F> {
        self.minifiers[self.minifiers.len() - 1]
            .circuit_fingerprint
            .clone()
    }
    pub fn get_common_data(&self) -> &CommonCircuitData<F, D> {
        &self.minifiers[self.minifiers.len() - 1].circuit_data.common
    }
    pub fn get_into_circuit_data(self) -> CircuitData<F, C, D> {
        self.minifiers.into_iter().last().unwrap().circuit_data
    }

    pub fn get_verifier_data(&self) -> &VerifierOnlyCircuitData<C, D> {
        &self.minifiers[self.minifiers.len() - 1]
            .circuit_data
            .verifier_only
    }
    pub fn verify(&self, proof: ProofWithPublicInputs<F, C, D>) -> Result<(), anyhow::Error> {
        self.minifiers[self.minifiers.len() - 1]
            .circuit_data
            .verify(proof)
    }
}
