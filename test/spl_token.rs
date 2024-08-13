//https://0xksure.medium.com/mint-tokens-on-solana-using-the-rust-sdk-3b05b07ca842

use solana_program_test::{tokio, ProgramTest};
use solana_sdk::program_pack::Pack;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;
use spl_token::state::{Account, Mint};
use spl_token::{id, instruction};

fn main() {}

#[tokio::test]
async fn test_initialize_mint() {
    let program_test = ProgramTest::default();
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let mint_account = Keypair::new();
    let owner = Keypair::new();
    let token_program = &id();

    let rent = banks_client.get_rent().await.expect("get rent");
    let mint_rent = rent.minimum_balance(Mint::LEN);

    // create mint_account
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
        &owner.pubkey(),
        None,
        9,
    )
    .expect("token mint instruction");

    let token_mint_tx = Transaction::new_signed_with_payer(
        &[token_mint_account_ix, token_mint_ix],
        Some(&payer.pubkey()),
        &[&payer, &mint_account],
        recent_blockhash,
    );

    banks_client
        .process_transaction(token_mint_tx)
        .await
        .expect("send token mint tx");

    // create account to hold the newly minted tokens
    let account_rent = rent.minimum_balance(Account::LEN);
    let token_account = Keypair::new();
    let new_token_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &token_account.pubkey(),
        account_rent,
        Account::LEN as u64,
        token_program,
    );
    let my_account = Keypair::new();
    let initialize_account_ix = instruction::initialize_account(
        token_program,
        &token_account.pubkey(),
        &mint_account.pubkey(),
        &my_account.pubkey(),
    )
    .expect("initialize token account");
    let create_new_token_account_tx = Transaction::new_signed_with_payer(
        &[new_token_account_ix, initialize_account_ix],
        Some(&payer.pubkey()),
        &[&payer, &token_account],
        recent_blockhash,
    );

    banks_client
        .process_transaction(create_new_token_account_tx)
        .await
        .expect("process create_new_token_account_tx");

    // mint to newly created account

    let mint_amount = 10u64;
    let mint_to_ix = instruction::mint_to(
        token_program,
        &mint_account.pubkey(),
        &token_account.pubkey(),
        &owner.pubkey(),
        &[],
        mint_amount,
    )
    .expect("mint ix");
    let mint_to_tx = Transaction::new_signed_with_payer(
        &[mint_to_ix],
        Some(&payer.pubkey()),
        &[&payer, &owner],
        recent_blockhash,
    );
    banks_client.process_transaction(mint_to_tx).await.expect("process mint to tx");


    // inspect account
    let token_account_info = banks_client.get_account(token_account.pubkey().clone()).await.unwrap();
    let account_data = Account::unpack(&token_account_info.data).unwrap();
    println!("account: data: {:?}", account_data);

    assert_eq!(account_data.amount, mint_amount, "not correct amount");
}
