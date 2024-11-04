use sol_rs::consts::MNEMONIC_CODE;
use sol_rs::keypair_generator::KeypairGenerator;
use sol_rs::network::Network;
use sol_rs::rpc_client::SolRpcClient;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::signer::Signer;

fn main() -> eyre::Result<()> {
    // 为了简单，我都用一份助记词，去生成 无限对keypair
    // 从 同一份私钥的 编号1(0被用了)， 给2,3,4,5,6转sol
    let rpc_client = SolRpcClient::new_with_network(Network::Devnet);

    let hash =
        rpc_client.ditribute_sol_to_idxs(1, &[2, 3, 4, 5, 6], &[LAMPORTS_PER_SOL / 10; 5])?;
    println!("tx hash: {hash}");

    // or
    let src = KeypairGenerator::get_keypair_with(MNEMONIC_CODE, 1);
    let to_pubkeys = (2..=6)
        .into_iter()
        .map(|idx| KeypairGenerator::get_keypair_with(MNEMONIC_CODE, idx).pubkey())
        .collect::<Vec<_>>();
    let hash =
        rpc_client.ditribute_sol_to_pubkeys(&src, &to_pubkeys, &[LAMPORTS_PER_SOL / 10; 5])?;
    println!("tx hash: {hash}");

    Ok(())
}
