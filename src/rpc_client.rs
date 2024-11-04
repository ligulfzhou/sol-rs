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
use spl_token::{
    id,
    state::{Account, Mint},
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
}

/// spl related
///
/// https://0xksure.medium.com/mint-tokens-on-solana-using-the-rust-sdk-3b05b07ca842
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

        let token_mint_ix = spl_token::instruction::initialize_mint(
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
        let mint_to_ix = spl_token::instruction::mint_to(
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

    pub fn transfer_spl_token(
        &self,
        payer: &Keypair,
        mint_account: &Pubkey,
        src_account: &Keypair,
        to_account: &Pubkey,
        amount: u64,
    ) -> eyre::Result<String> {
        let token_program = &id();
        let src_ata = spl_associated_token_account::get_associated_token_address(
            &src_account.pubkey(),
            mint_account,
        );
        let to_ata =
            spl_associated_token_account::get_associated_token_address(to_account, mint_account);
        let mut instructions = vec![];
        if self.rpc_client.get_account(&to_ata).is_err() {
            let assoc_ix =
                spl_associated_token_account::instruction::create_associated_token_account(
                    &payer.pubkey(),
                    &to_account,
                    mint_account,
                    &id(),
                );
            instructions.push(assoc_ix);
        }
        instructions.push(spl_token::instruction::transfer(
            token_program,
            &src_ata,
            &to_ata,
            &src_account.pubkey(),
            &[&payer.pubkey(), &src_account.pubkey()],
            amount,
        )?);

        self.send_tx(instructions, &[src_account, payer], &payer.pubkey())
    }

    // not usefull
    pub fn transfer_spl_token_to_ata(
        &self,
        payer: &Keypair,
        mint_account: &Pubkey,
        src_account: &Keypair,
        to_ata: &Pubkey,
        amount: u64,
    ) -> eyre::Result<String> {
        let token_program = &id();
        let src_ata = spl_associated_token_account::get_associated_token_address(
            &src_account.pubkey(),
            mint_account,
        );
        let transfer_ix = spl_token::instruction::transfer(
            token_program,
            &src_ata,
            &to_ata,
            &src_account.pubkey(),
            &[&payer.pubkey(), &src_account.pubkey()],
            amount,
        )?;

        // todo: have not apply to other places
        // src_account and payer may be same.
        // though [payer, payer] still works
        let signers = {
            if src_account.pubkey().eq(&payer.pubkey()) {
                vec![payer]
            } else {
                vec![src_account, payer]
            }
        };

        self.send_tx(vec![transfer_ix], &signers, &payer.pubkey())
    }

    pub fn distribute_spl_token_to_idxs(
        &self,
        payer: &Keypair,
        mint_account: &Pubkey,
        src_account_idx: u32,
        to_account_idxs: &[u32],
        amounts: &[u64],
    ) -> eyre::Result<String> {
        let src_account = KeypairGenerator::get_keypair_with(MNEMONIC_CODE, src_account_idx);
        let to_accounts = to_account_idxs
            .iter()
            .map(|idx| KeypairGenerator::get_keypair_with(MNEMONIC_CODE, *idx).pubkey())
            .collect::<Vec<_>>();

        self.distribute_spl_token_to_pubkeys(
            payer,
            mint_account,
            &src_account,
            &to_accounts,
            amounts,
        )
    }

    pub fn distribute_spl_token_to_pubkeys(
        &self,
        payer: &Keypair,
        mint_account: &Pubkey,
        src_account: &Keypair,
        to_accounts: &[Pubkey],
        amounts: &[u64],
    ) -> eyre::Result<String> {
        let token_program = &id();

        // src_ata
        let src_ata = spl_associated_token_account::get_associated_token_address(
            &src_account.pubkey(),
            mint_account,
        );

        // check atas exists
        let atas = to_accounts
            .iter()
            .map(|account| {
                spl_associated_token_account::get_associated_token_address(account, mint_account)
            })
            .collect::<Vec<_>>();
        let ata_accounts = self.rpc_client.get_multiple_accounts(&atas)?;

        let mut instructions = vec![];
        // if ata not exists, create it
        for (idx, account) in ata_accounts.iter().enumerate() {
            if account.is_some() {
                continue;
            }
            instructions.push(
                spl_associated_token_account::instruction::create_associated_token_account(
                    &payer.pubkey(),
                    &to_accounts[idx],
                    mint_account,
                    &token_program,
                ),
            );
        }

        for (idx, ata) in atas.iter().enumerate() {
            instructions.push(spl_token::instruction::transfer(
                token_program,
                &src_ata,
                &ata,
                &src_account.pubkey(),
                &[&payer.pubkey(), &src_account.pubkey()],
                amounts[idx],
            )?);
        }

        self.send_tx(instructions, &[src_account, payer], &payer.pubkey())
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
