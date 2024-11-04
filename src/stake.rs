use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    stake::{
        self,
        state::{Authorized, Lockup},
    },
    transaction::Transaction,
};
use std::str::FromStr;

pub struct Stake {
    rpc_client: RpcClient,
    payer: Keypair,
    stake_account: Keypair,
}

impl Stake {
    pub fn new(url: &str, payer: Keypair, stake_account: Keypair) -> Self {
        let rpc_client = RpcClient::new(url);

        Self {
            rpc_client,
            payer,
            stake_account,
        }
    }

    pub fn get_validators(&self) -> (Pubkey, Vec<Pubkey>) {
        let vote_accounts = self
            .rpc_client
            .get_vote_accounts()
            .expect("get vote accounts");

        let current = vote_accounts.current.clone();
        let current_pubkey = Pubkey::from_str(&current[0].vote_pubkey).expect("parse vote pubkey");
        let mut all = vote_accounts.delinquent;
        all.extend(vote_accounts.current);

        let all_pubkeys = all
            .into_iter()
            .map(|info| Pubkey::from_str(&info.vote_pubkey).expect("parse vote pubkey"))
            .collect::<Vec<_>>();

        (current_pubkey, all_pubkeys)
    }

    pub fn create_stake_account(
        &self,
        amount_to_stake: u64,
        withdraw_ts: i64,
        withdraw_epoch: u64,
    ) -> eyre::Result<()> {
        let blockhash = self.rpc_client.get_latest_blockhash()?;

        let create_stake_account_stake_ix = stake::instruction::create_account(
            &self.payer.pubkey(),
            &self.stake_account.pubkey(),
            &Authorized {
                staker: self.payer.pubkey(),
                withdrawer: self.payer.pubkey(),
            },
            &Lockup {
                unix_timestamp: withdraw_ts,
                epoch: withdraw_epoch,
                custodian: self.payer.pubkey(),
            },
            amount_to_stake,
        );
        let tx = Transaction::new_signed_with_payer(
            &create_stake_account_stake_ix,
            Some(&self.payer.pubkey()),
            &[&self.payer, &self.stake_account],
            blockhash,
        );
        let sig = self.rpc_client.send_and_confirm_transaction(&tx)?;
        println!("sig: {:?}", sig);

        Ok(())
    }

    pub fn stake(&self, validator_pubkey: Pubkey) -> eyre::Result<()> {
        let blockhash = self.rpc_client.get_latest_blockhash()?;

        // let validator_pubkey = Pubkey::from_str("28rDknpdBPNu5RU9yxbVqqHwnbXB9qaCigw1M53g7Nps")?;
        let stake_ix = stake::instruction::delegate_stake(
            &self.stake_account.pubkey(),
            &self.payer.pubkey(),
            &validator_pubkey,
        );
        let tx = Transaction::new_signed_with_payer(
            &[stake_ix],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            blockhash,
        );
        let sig = self.rpc_client.send_and_confirm_transaction(&tx)?;
        print!("sig: {:?}", sig);

        Ok(())
    }

    pub fn check_stake_actived(&self, validator_pubkey: Pubkey) -> eyre::Result<bool> {
        Ok(true)
    }
}
