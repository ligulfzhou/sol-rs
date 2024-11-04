use solana_sdk::derivation_path::DerivationPath;
use solana_sdk::signature::{
    generate_seed_from_seed_phrase_and_passphrase, keypair_from_seed_and_derivation_path, Keypair,
};

pub struct KeypairGenerator;

impl KeypairGenerator {
    pub fn get_keypair_with(phrase: &str, at_index: u32) -> Keypair {
        let seed = generate_seed_from_seed_phrase_and_passphrase(phrase, "");
        let derivation_path = DerivationPath::new_bip44(Some(at_index), Some(0));
        keypair_from_seed_and_derivation_path(&seed, Some(derivation_path)).unwrap()
    }

    pub fn random() -> Keypair {
        Keypair::new()
    }
}
