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
}

fn create_stake_account() -> anyhow::Result<()> {
    let url = "https://api.devnet.solana.com".to_string();
    let rpc_client = RpcClient::new(url);
    let blockhash = rpc_client.get_latest_blockhash()?;

    let owner = Keypair::from_bytes(&[
        11, 191, 140, 10, 171, 104, 130, 22, 77, 157, 200, 254, 89, 207, 201, 107, 80, 169, 106,
        79, 62, 168, 173, 57, 208, 203, 74, 159, 115, 69, 133, 195, 155, 0, 125, 68, 208, 75, 27,
        10, 82, 94, 151, 44, 75, 72, 248, 10, 148, 71, 133, 225, 98, 39, 184, 136, 161, 30, 24,
        175, 20, 69, 96, 158,
    ])?;
    let stake_account = MyAccount::get_keypair_with(mnemonic, 1000);
    let amount_to_stake = 100_000_000u64;
    let create_stake_account_stake_ix = stake::instruction::create_account(
        &owner.pubkey(),
        &stake_account.pubkey(),
        &Authorized {
            staker: owner.pubkey(),
            withdrawer: owner.pubkey(),
        },
        &Lockup {
            unix_timestamp: 0,
            epoch: 0,
            custodian: owner.pubkey(),
        },
        amount_to_stake,
    );
    let tx = Transaction::new_signed_with_payer(
        &create_stake_account_stake_ix,
        Some(&owner.pubkey()),
        &[&owner, &stake_account.0],
        blockhash,
    );
    let sig = rpc_client.send_and_confirm_transaction(&tx)?;
    println!("sig: {:?}", sig);

    Ok(())
}

fn stake() -> anyhow::Result<()> {
    let url = "https://api.devnet.solana.com".to_string();
    let rpc_client = RpcClient::new(url);
    let blockhash = rpc_client.get_latest_blockhash()?;

    let owner = Keypair::from_bytes(&[
        11, 191, 140, 10, 171, 104, 130, 22, 77, 157, 200, 254, 89, 207, 201, 107, 80, 169, 106,
        79, 62, 168, 173, 57, 208, 203, 74, 159, 115, 69, 133, 195, 155, 0, 125, 68, 208, 75, 27,
        10, 82, 94, 151, 44, 75, 72, 248, 10, 148, 71, 133, 225, 98, 39, 184, 136, 161, 30, 24,
        175, 20, 69, 96, 158,
    ])?;
    let stake_account = MyAccount::get_keypair_with(mnemonic, 1000);
    let validator_pubkey = Pubkey::from_str("28rDknpdBPNu5RU9yxbVqqHwnbXB9qaCigw1M53g7Nps")?;

    let stake_ix = stake::instruction::delegate_stake(
        &stake_account.pubkey(),
        &owner.pubkey(),
        &validator_pubkey,
    );
    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&owner.pubkey()),
        &[&owner],
        blockhash,
    );
    let sig = rpc_client.send_and_confirm_transaction(&tx)?;
    print!("sig: {:?}", sig);

    Ok(())
}

fn check_stake_actived() -> anyhow::Result<()> {
    let url = "https://api.devnet.solana.com".to_string();
    let rpc_client = RpcClient::new(url);
    let blockhash = rpc_client.get_latest_blockhash()?;

    let owner = Keypair::from_bytes(&[
        11, 191, 140, 10, 171, 104, 130, 22, 77, 157, 200, 254, 89, 207, 201, 107, 80, 169, 106,
        79, 62, 168, 173, 57, 208, 203, 74, 159, 115, 69, 133, 195, 155, 0, 125, 68, 208, 75, 27,
        10, 82, 94, 151, 44, 75, 72, 248, 10, 148, 71, 133, 225, 98, 39, 184, 136, 161, 30, 24,
        175, 20, 69, 96, 158,
    ])?;
    let stake_account = MyAccount::get_keypair_with(mnemonic, 1000);
    let validator_pubkey = Pubkey::from_str("28rDknpdBPNu5RU9yxbVqqHwnbXB9qaCigw1M53g7Nps")?;

    let stake_ix = stake::instruction::delegate_stake(
        &stake_account.pubkey(),
        &owner.pubkey(),
        &validator_pubkey,
    );
    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&owner.pubkey()),
        &[&owner],
        blockhash,
    );
    let sig = rpc_client.send_and_confirm_transaction(&tx)?;
    print!("sig: {:?}", sig);

    Ok(())
}

fn main() {
    create_stake_account().expect("");

    stake().expect("");
}
