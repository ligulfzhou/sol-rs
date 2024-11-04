use crate::consts::MNEMONIC_CODE;
use crate::keypair_generator::KeypairGenerator;
use crate::network::Network;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_token::state::Account;
use spl_token::{id, instruction, state::Mint};
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

/// spl related
///
/// create_spl_token
impl SolRpcClient {
    pub fn create_spl_token(
        &self,
        payer: &Keypair,
        mint_account: &Keypair,
    ) -> eyre::Result<String> {
        let token_program = &id();

        let mint_rent = self
            .rpc_client
            .get_minimum_balance_for_rent_exemption(Mint::LEN)?;

        let token_mint_account_ix = system_instruction::create_account(
            &payer.pubkey(),
            &mint_account.pubkey(),
            mint_rent,
            Mint::LEN as u64,
            token_program,
        );

        let token_mint_ix = instruction::initialize_mint(
            token_program,
            &mint_account.pubkey(),
            &payer.pubkey(),
            None,
            9,
        )?;

        self.send_tx(
            vec![token_mint_account_ix, token_mint_ix],
            &[payer, mint_account],
            &payer.pubkey(),
        )
    }

    pub fn mint_spl_token(
        &self,
        payer: &Keypair,
        mint_account: &Keypair,
        to_account: &Pubkey,
        amount: u64,
    ) -> eyre::Result<String> {
        let associated_token_account = spl_associated_token_account::get_associated_token_address(
            to_account,
            &mint_account.pubkey(),
        );

        let mut instructions = vec![];
        // create ATA if not exists.
        if self
            .rpc_client
            .get_account(&associated_token_account)
            .is_err()
        {
            let assoc_ix =
                spl_associated_token_account::instruction::create_associated_token_account(
                    &payer.pubkey(),
                    &to_account,
                    &mint_account.pubkey(),
                    &id(),
                );

            instructions.push(assoc_ix);
        }

        // mint spl token to PDA
        let mint_to_ix = instruction::mint_to(
            &id(),
            &mint_account.pubkey(),
            &associated_token_account,
            &payer.pubkey(),
            &[&mint_account.pubkey(), &payer.pubkey()],
            amount,
        )?;
        instructions.push(mint_to_ix);

        self.send_tx(instructions, &[mint_account, payer], &payer.pubkey())
    }

    pub fn get_spl_token_balance(
        &self,
        mint_account: &Pubkey,
        holder: &Pubkey,
    ) -> eyre::Result<u64> {
        let ata = spl_associated_token_account::get_associated_token_address(holder, mint_account);

        self.get_spl_token_balance_of_ata(&ata)
    }

    pub fn get_spl_token_balance_of_ata(&self, ata: &Pubkey) -> eyre::Result<u64> {
        let account = self.rpc_client.get_account(ata)?;
        Ok(Account::unpack(&account.data)?.amount)
    }

    pub fn transfer_spl_token() {}
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
    use super::*;
    const PAYER_IDX: u32 = 1;
    const MINT_ACCOUNT_IDX: u32 = 99;

    #[test]
    fn test_create_spl_token() -> eyre::Result<()> {
        let rpc_client = SolRpcClient::new_with_network(Network::Devnet);

        let payer = KeypairGenerator::get_keypair_with(MNEMONIC_CODE, PAYER_IDX);
        let mint_account = KeypairGenerator::get_keypair_with(MNEMONIC_CODE, MINT_ACCOUNT_IDX);

        // create mint account.
        let hash = rpc_client.create_spl_token(&payer, &mint_account)?;
        println!("create spl_token: {:?}", hash);

        Ok(())
    }

    #[test]
    fn test_mint_spl_token() -> eyre::Result<()> {
        let rpc_client = SolRpcClient::new_with_network(Network::Devnet);

        let payer = KeypairGenerator::get_keypair_with(MNEMONIC_CODE, PAYER_IDX);
        let mint_account = KeypairGenerator::get_keypair_with(MNEMONIC_CODE, MINT_ACCOUNT_IDX);

        // mint spl_token
        let hash = rpc_client.mint_spl_token(
            &payer,
            &mint_account,
            &payer.pubkey(),
            10_0000_0000 * 10_0000_0000,
        )?;
        println!("mint_spl_token: {:?}", hash);

        Ok(())
    }

    #[test]
    fn test_fetch_spl_token_balance() -> eyre::Result<()> {
        let rpc_client = SolRpcClient::new_with_network(Network::Devnet);

        let payer = KeypairGenerator::get_keypair_with(MNEMONIC_CODE, PAYER_IDX);
        let mint_account = KeypairGenerator::get_keypair_with(MNEMONIC_CODE, MINT_ACCOUNT_IDX);

        let ata = spl_associated_token_account::get_associated_token_address(
            &payer.pubkey(),
            &mint_account.pubkey(),
        );
        println!("ata: address: {:?}", ata.to_string());
        let balance = rpc_client.get_spl_token_balance_of_ata(&ata)?;
        println!("spl_token ata balance: {:?}", balance);

        let balance = rpc_client.get_spl_token_balance(&mint_account.pubkey(), &payer.pubkey())?;
        println!("spl_token balance: {:?}", balance);

        Ok(())
    }

    #[test]
    fn test_distribute_spl_token() -> eyre::Result<()> {
        let rpc_client = SolRpcClient::new_with_network(Network::Devnet);

        let payer = KeypairGenerator::get_keypair_with(MNEMONIC_CODE, PAYER_IDX);
        let mint_account = KeypairGenerator::get_keypair_with(MNEMONIC_CODE, MINT_ACCOUNT_IDX);

        // mint spl_token
        let hash = rpc_client.mint_spl_token(
            &payer,
            &mint_account,
            &payer.pubkey(),
            10_0000_0000 * 10_0000_0000,
        )?;
        println!("mint_spl_token: {:?}", hash);

        Ok(())
    }
}
