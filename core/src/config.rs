use serde::{Deserialize, Serialize};
use std::{fs, env};
use std::path::PathBuf;
use crate::contracts::token::Token;
use crate::contracts::multicall::Multicall;
use crate::contracts::global_fetch::GlobalFetch;
use wasm_bindgen::prelude::*;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub selected_account: Option<String>,
    pub chain: Chain,
    pub tokens: Vec<Token>,
    pub contract_address: ContractAddress
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Chain {
    pub chain_id: u64,
    pub rpc_urls: Vec<String>,
    pub multicall_address: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ContractAddress {
    pub vault: String,
    pub plp_manager: String,
    pub plp_token: String,
}

impl Config {
    pub fn set_selected_account(&mut self, account: String) {
        self.selected_account = Some(account);
    }

}

pub fn load_config(chain_id: u64) -> Result<Config, String> {
    // TODO: This macro in Rust allows you to include the contents of a file as a string in your compiled binary at compile time.
    // So we need change to fetch from server so that we can remote config the data, especially in
    // in mobile apps
    let config_str = include_str!("../conf/bsc_97.toml");
    let config: Config = toml::from_str(&config_str).expect("Failed to parse TOML");
    Ok(config)
}

// write test for load_config

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_config_works() {
        let config = load_config(97).unwrap();
        println!("Loaded config: {:?}", config);
        assert_eq!(config.chain.chain_id, 97);
    }
}

