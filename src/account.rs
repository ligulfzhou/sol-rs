use bip39::Mnemonic;
use solana_client::rpc_client::RpcClient;
use solana_sdk::transaction::Transaction;
use solana_sdk::{
    derivation_path::DerivationPath,
    pubkey::Pubkey,
    signature::{
        generate_seed_from_seed_phrase_and_passphrase, keypair_from_seed_and_derivation_path,
        Keypair, Signature, Signer,
    },
    system_instruction,
};
use std::str::FromStr;

pub struct MyAccount(pub Keypair);

impl MyAccount {
    pub fn generate_mnemonic_code(word_count: usize) -> String {
        Mnemonic::generate(word_count)
            .expect("word count not valid")
            .to_string()
    }

    pub fn get_keypair_with(phrase: &str, at_index: u32) -> MyAccount {
        let seed = generate_seed_from_seed_phrase_and_passphrase(phrase, "");
        let derivation_path = DerivationPath::new_bip44(Some(at_index), Some(0));
        MyAccount(keypair_from_seed_and_derivation_path(&seed, Some(derivation_path)).expect(""))
    }

    pub fn random() -> MyAccount {
        MyAccount(Keypair::new())
    }
}

impl MyAccount {
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

impl MyAccount {
    pub fn sign(&self, msg: String) -> String {
        self.0.sign_message(&msg.as_bytes()).to_string()
    }

    pub fn verify_sig(&self, sig: String, msg: String) -> anyhow::Result<bool> {
        let signature = Signature::from_str(&sig).expect("invalid signature");
        Ok(signature.verify(&self.pubkey().as_ref(), &msg.as_bytes()))
    }
}

impl MyAccount {
    pub fn from_bytes(bs: &[u8]) -> anyhow::Result<MyAccount> {
        Ok(MyAccount(Keypair::from_bytes(bs)?))
    }

    pub fn from_base58_str(str: &str) -> Self {
        MyAccount(Keypair::from_base58_string(str))
    }
}

impl MyAccount {
    pub fn send_sol(&self, to: Pubkey, lamports: u64) -> anyhow::Result<()> {
        let url = "https://api.devnet.solana.com".to_string();
        let rpc_client = RpcClient::new(url);

        let blockhash = rpc_client.get_latest_blockhash()?;

        let transfer_ix = system_instruction::transfer(&self.0.pubkey(), &to, lamports);
        let transfer_tx = Transaction::new_signed_with_payer(
            &[transfer_ix],
            Some(&self.0.pubkey()),
            &[&self.0],
            blockhash,
        );

        let signature = rpc_client.send_transaction(&transfer_tx)?;
        let statuses = rpc_client.get_signature_statuses(&[signature])?.value;
        println!("status: {:?}", statuses);

        Ok(())
    }
}
