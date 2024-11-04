use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    derivation_path::DerivationPath,
    pubkey::Pubkey,
    signature::{
        generate_seed_from_seed_phrase_and_passphrase, keypair_from_seed_and_derivation_path,
        Keypair, Signature, Signer,
    },
    system_instruction,
    transaction::Transaction,
};
use std::str::FromStr;

#[derive(Debug)]
pub struct SolAccount(pub Keypair);

impl SolAccount {
    pub fn get_keypair_with(phrase: &str, at_index: u32) -> SolAccount {
        let seed = generate_seed_from_seed_phrase_and_passphrase(phrase, "");
        let derivation_path = DerivationPath::new_bip44(Some(at_index), Some(0));
        SolAccount(keypair_from_seed_and_derivation_path(&seed, Some(derivation_path)).expect(""))
    }

    pub fn random() -> SolAccount {
        SolAccount(Keypair::new())
    }
}

impl SolAccount {
    pub fn keypair(&self) -> &Keypair {
        &self.0
    }
    pub fn address(&self) -> String {
        self.0.pubkey().to_string()
    }
    pub fn pubkey(&self) -> Pubkey {
        self.0.pubkey()
    }
}