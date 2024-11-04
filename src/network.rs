use crate::consts::{DEVNET_RPC, MAINNET_RPC, TESTNET_RPC};

#[derive(Debug)]
pub enum Network {
    Testnet,
    Devnet,
    Mainnet,
}
impl Network {
    pub fn to_host(&self) -> String {
        match self {
            Network::Testnet => TESTNET_RPC.to_string(),
            Network::Devnet => DEVNET_RPC.to_string(),
            Network::Mainnet => MAINNET_RPC.to_string(),
        }
    }
}
