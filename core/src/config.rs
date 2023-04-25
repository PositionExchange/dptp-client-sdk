use serde::{Deserialize, Serialize};
use crate::contracts::token::Token;
use crate::contracts::multicall::Multicall;
use crate::contracts::global_fetch::GlobalFetch;
use wasm_bindgen::prelude::*;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub selected_account: Option<String>,
    pub chain: Chain,
    pub tokens: Vec<Token>,
    pub contract_address: ContractAddress,
    pub contract_spender : Vec<Spender>
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Spender {
    pub address: String,
    pub name: String,
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
    pub reward_router: String,
    pub futurx_gateway : String,
    pub reward_tracker_fee_plp : String,
    pub vester_plp :String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ContractSpenderAddress {
    pub plp_manager: String,
}

impl Config {
    pub fn set_selected_account(&mut self, account: String) {
        self.selected_account = Some(account);
    }

    pub fn get_token_by_token_address(&self, token_address: String) -> Option<Token> {
        let mut token: Option<Token> = None;
        for t in self.tokens.iter() {
            if t.address == token_address {
                token = Some(t.clone());
                break;
            }
        }
        token
    }

}

pub fn load_config(chain_id: u64) -> Result<Config, String> {
    // TODO: This macro in Rust allows you to include the contents of a file as a string in your compiled binary at compile time.
    // So we need change to fetch from server so that we can remote config the data, especially in
    // in mobile apps

    let mut config_str = include_str!("../conf/bsc_97.toml");

    match chain_id {
        97 => config_str = include_str!("../conf/bsc_97.toml"),
        56 => config_str = include_str!("../conf/bsc_56.toml"),
        421613 => config_str = include_str!("../conf/arb_421613.toml"),
        42161 => config_str = include_str!("../conf/arb_42161.toml"),
        _ => {}
    }
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

