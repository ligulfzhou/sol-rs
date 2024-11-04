use crate::consts::MNEMONIC_CODE;
use crate::keypair_generator::KeypairGenerator;
use crate::network::Network;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct SolRpcClient {
    pub rpc_client: Arc<RpcClient>,
}

impl SolRpcClient {
    pub fn new(rpc_client: Arc<RpcClient>) -> Self {
        Self { rpc_client }
    }

    pub fn new_with_rpc_host(rpc_host: String) -> Self {
        let rpc_client = RpcClient::new(rpc_host);
        Self::new(Arc::new(rpc_client))
    }

    pub fn new_with_network(network: Network) -> Self {
        Self::new_with_rpc_host(network.to_host())
    }
}

impl SolRpcClient {
    pub fn ditribute_sol_to_idxs(
        &self,
        from_idx: u32,
        idxs: &[u32],
        amounts: &[u64],
    ) -> eyre::Result<String> {
        let from_keypair = KeypairGenerator::get_keypair_with(MNEMONIC_CODE, from_idx);
        let pubkeys = idxs
            .iter()
            .map(|idx| KeypairGenerator::get_keypair_with(MNEMONIC_CODE, *idx).pubkey())
            .collect::<Vec<_>>();

        self.ditribute_sol_to_pubkeys(&from_keypair, &pubkeys, amounts)
    }

    pub fn ditribute_sol_to_pubkeys(
        &self,
        from_keypair: &Keypair,
        to_pubkeys: &[Pubkey],
        amounts: &[u64],
    ) -> eyre::Result<String> {
        let pubkey_amounts = to_pubkeys
            .to_vec()
            .into_iter()
            .zip(amounts.to_vec().into_iter())
            .collect::<Vec<_>>();
        let ix = system_instruction::transfer_many(&from_keypair.pubkey(), &pubkey_amounts);
        self.send_tx(ix, &[from_keypair], &from_keypair.pubkey())
    }

    pub fn send_tx(
        &self,
        instructions: Vec<Instruction>,
        signers: &[&Keypair],
        payer: &Pubkey,
    ) -> eyre::Result<String> {
        let recent_hash = self.rpc_client.get_latest_blockhash()?;

        let tx =
            Transaction::new_signed_with_payer(&instructions, Some(&payer), signers, recent_hash);
        let sig = self.rpc_client.send_and_confirm_transaction(&tx)?;

        Ok(sig.to_string())
    }
}

impl From<Arc<RpcClient>> for SolRpcClient {
    fn from(value: Arc<RpcClient>) -> Self {
        Self::new(value)
    }
}

impl From<String> for SolRpcClient {
    fn from(value: String) -> Self {
        Self::new_with_rpc_host(value)
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_distribute() {}
}
