use plonky2::hash::hash_types::RichField;

use crate::{
    actors::{
        rpc_processor::{CityScenarioRequestedActionsFromRPC, QRPCProcessor},
        traits::OrchestratorRPCEventSenderSync,
    },
    api::data::block::rpc_request::{
        CityAddWithdrawalRPCRequest, CityClaimDepositRPCRequest, CityRegisterUserRPCRequest,
        CityTokenTransferRPCRequest,
    },
    qworker::proof_store::QProofStore,
};

pub struct SimpleCoordinatatorRPCQueueMemory<F: RichField> {
    pub claim_l1_deposits: Vec<CityClaimDepositRPCRequest>,
    pub register_users: Vec<CityRegisterUserRPCRequest<F>>,
    pub add_withdrawals: Vec<CityAddWithdrawalRPCRequest>,
    pub token_transfers: Vec<CityTokenTransferRPCRequest>,
}
impl<F: RichField> SimpleCoordinatatorRPCQueueMemory<F> {
    pub fn new() -> Self {
        Self {
            claim_l1_deposits: Vec::new(),
            register_users: Vec::new(),
            add_withdrawals: Vec::new(),
            token_transfers: Vec::new(),
        }
    }
    pub fn get_requested_actions_from_rpc<PS: QProofStore>(
        &self,
        proof_store: &mut PS,
        checkpoint_id: u64,
    ) -> anyhow::Result<CityScenarioRequestedActionsFromRPC<F>> {
        let mut rpc_processor = QRPCProcessor::new(checkpoint_id);
        rpc_processor.process_register_users(0, &self.register_users)?;
        rpc_processor.process_deposits(proof_store, 0, &self.claim_l1_deposits)?;
        rpc_processor.process_transfers(proof_store, 0, &self.token_transfers)?;
        rpc_processor.process_withdrawals(proof_store, 0, &self.add_withdrawals)?;
        Ok(rpc_processor.output)
    }
}
impl<F: RichField> OrchestratorRPCEventSenderSync<F> for SimpleCoordinatatorRPCQueueMemory<F> {
    fn notify_rpc_claim_deposit(
        &mut self,
        event: &CityClaimDepositRPCRequest,
    ) -> anyhow::Result<()> {
        self.claim_l1_deposits.push(event.clone());
        Ok(())
    }

    fn notify_rpc_register_user(
        &mut self,
        event: &CityRegisterUserRPCRequest<F>,
    ) -> anyhow::Result<()> {
        self.register_users.push(event.clone());
        Ok(())
    }

    fn notify_rpc_add_withdrawal(
        &mut self,
        event: &CityAddWithdrawalRPCRequest,
    ) -> anyhow::Result<()> {
        self.add_withdrawals.push(event.clone());
        Ok(())
    }

    fn notify_rpc_token_transfer(
        &mut self,
        event: &CityTokenTransferRPCRequest,
    ) -> anyhow::Result<()> {
        self.token_transfers.push(event.clone());
        Ok(())
    }
}
