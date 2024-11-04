use crate::account::SolAccount;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

use spl_token::{
    id, instruction,
    state::{Account, Mint},
};

pub struct SPLToken;

impl SPLToken {
    pub fn ata_address(token_mint_account: &Pubkey, account: &Pubkey) -> Pubkey {
        spl_associated_token_account::get_associated_token_address(account, token_mint_account)
    }

    /// transfer spl_token to
    pub fn tranfer_spl_token(
        from_account: &SolAccount,
        to_account: &SolAccount,
        amount: u64,
        payer: &SolAccount,
    ) -> eyre::Result<()> {
        todo!()
    }
}

pub struct SplToken {
    rpc_client: RpcClient,
    payer: Keypair,
    mint_account: Keypair,
}

impl SplToken {
    pub fn new(url: &str, payer: Keypair, mint_account: Keypair) -> Self {
        let rpc_client = RpcClient::new(url);

        Self {
            rpc_client,
            payer,
            mint_account,
        }
    }

    /// create spl token
    pub fn create_spl_token(&self) -> eyre::Result<()> {
        let blockhash = self.rpc_client.get_latest_blockhash()?;

        let token_program = &id();

        let mint_rent = self
            .rpc_client
            .get_minimum_balance_for_rent_exemption(Mint::LEN)?;

        let token_mint_account_ix = system_instruction::create_account(
            &self.payer.pubkey(),
            &self.mint_account.pubkey(),
            mint_rent,
            Mint::LEN as u64,
            token_program,
        );
        let token_mint_ix = instruction::initialize_mint(
            token_program,
            &self.mint_account.pubkey(),
            &self.payer.pubkey(),
            None,
            9,
        )?;

        let mint_tx = Transaction::new_signed_with_payer(
            &[token_mint_account_ix, token_mint_ix],
            Some(&self.payer.pubkey()),
            &[&self.payer, &self.mint_account],
            blockhash,
        );

        let sig = self.rpc_client.send_and_confirm_transaction(&mint_tx)?;
        print!("sig: {:?}", sig);

        Ok(())
    }

    /// get spl token balance.
    pub fn get_spl_token_balance(&self, holder: Pubkey) -> eyre::Result<u64> {
        let associated_token_account = self.get_existing_or_create_ata(holder)?;
        let account = self.rpc_client.get_account(&associated_token_account)?;
        let data = Account::unpack(&account.data).expect("unpack account.data");

        Ok(data.amount)
    }

    /// get ATA account of receiver, if not exists, create it.
    pub fn get_existing_or_create_ata(&self, receiver: Pubkey) -> eyre::Result<Pubkey> {
        // ata account of receiver_account
        let associated_token_account = spl_associated_token_account::get_associated_token_address(
            &receiver,
            &self.mint_account.pubkey(),
        );

        // create ATA if not exists.
        if self
            .rpc_client
            .get_account(&associated_token_account)
            .is_err()
        {
            let assoc_ix =
                spl_associated_token_account::instruction::create_associated_token_account(
                    &self.payer.pubkey(),
                    &receiver,
                    &self.mint_account.pubkey(),
                    &id(),
                );

            let blockhash = self.rpc_client.get_latest_blockhash()?;

            // Build the transaction
            let tx = Transaction::new_signed_with_payer(
                &[assoc_ix],
                Some(&self.mint_account.pubkey()),
                &[&self.mint_account],
                blockhash,
            );

            // Send the transaction
            self.rpc_client.send_and_confirm_transaction(&tx).unwrap();
        }

        Ok(associated_token_account)
    }

    /// mint {amount} of token to receiver
    pub fn mint_to(&self, receiver: Pubkey, amount: u64) -> eyre::Result<()> {
        // get or create PDA of receiver_account
        let associated_token_account = self.get_existing_or_create_ata(receiver)?;

        // mint spl token to PDA
        let mint_to_ix = instruction::mint_to(
            &id(),
            &self.mint_account.pubkey(),
            &associated_token_account,
            &self.payer.pubkey(),
            &[&self.payer.pubkey()],
            amount,
        )
        .expect("mint ix");

        let blockhash = self.rpc_client.get_latest_blockhash()?;
        let mint_tx = Transaction::new_signed_with_payer(
            &[mint_to_ix],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            blockhash,
        );

        let sig = self.rpc_client.send_and_confirm_transaction(&mint_tx)?;
        print!("sig: {:?}", sig);

        Ok(())
    }
}
