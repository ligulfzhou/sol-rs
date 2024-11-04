use rand::Rng;
use sol_rs::consts::MNEMONIC_CODE;
use sol_rs::keypair_generator::KeypairGenerator;
use sol_rs::network::Network;
use sol_rs::rpc_client::SolRpcClient;
use solana_sdk::signer::Signer;

fn main() -> eyre::Result<()> {
    let client = SolRpcClient::new_with_network(Network::Devnet);

    // use idx#1 as payer, randomly pick one from (1000_0000, 2000_0000) as mint_account
    let payer = KeypairGenerator::get_keypair_with(MNEMONIC_CODE, 1);
    let mint_account_idx = rand::thread_rng().gen_range(1000_0000..2000_0000);
    println!("mint_account_idx: {}", mint_account_idx);
    let mint_account = KeypairGenerator::get_keypair_with(MNEMONIC_CODE, mint_account_idx);

    // create and initialize mint_account
    let hash = client.create_spl_token(&payer, &mint_account)?;
    println!("create spl_token: {hash}");

    // mint to payer or others
    let hash = client.mint_spl_token(
        &payer,
        &mint_account,
        &payer.pubkey(),
        1_0000_0000 * 10_0000_0000,
    )?;
    println!(
        "mint 1_0000_0000 spl_token to {:?}, hash: {:?}",
        payer.pubkey(),
        hash
    );

    // transfer spl_token
    let to_account = KeypairGenerator::get_keypair_with(MNEMONIC_CODE, 1000);
    let hash = client.transfer_spl_token(
        &payer,
        &mint_account.pubkey(),
        &payer,
        &to_account.pubkey(),
        10 * 10_0000_0000,
    )?;
    println!("transfer spl_token: {hash}");

    Ok(())
}
