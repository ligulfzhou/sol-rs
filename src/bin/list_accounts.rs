use sol_rs::consts::MNEMONIC_CODE;
use sol_rs::keypair_generator::KeypairGenerator;
use solana_sdk::signature::Signer;

fn main() {
    for idx in 0..100 {
        let keypair = KeypairGenerator::get_keypair_with(MNEMONIC_CODE, idx);
        println!(
            "#{idx}: private_key: {:?}, address: {:?}",
            keypair.to_base58_string(),
            keypair.pubkey().to_string()
        );
    }
}
