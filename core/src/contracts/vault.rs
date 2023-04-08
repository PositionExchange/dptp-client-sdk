use std::{ops::Div, str::FromStr};

use ethers::{
    types::{Address, Bytes, U256},
};
use rust_decimal::Decimal;
use tokio::sync::Mutex;

use crate::config::Chain;

use super::token::Token;
use super::multicall::*;



pub struct Vault {
    vault_addr: String,
    chain: Chain,
}

impl Vault {
    pub fn new(vault_addr: &String, chain: &Chain) -> Self {
        Self { vault_addr: vault_addr.to_string(), chain: chain.clone() }
    }
    pub async fn fetch_token_configuration(&self, tokens: &Mutex<Vec<Token>>) -> anyhow::Result<()> {
         let mut tokens = tokens.lock().await;
        let calls: Vec<(Address, Bytes)> = tokens.iter().map(|token| {
            let (vault_addr, data) = token.build_get_vault_token_configuration_call(&self.vault_addr);
            (vault_addr, data)
        }).collect();
        let results = self.chain.execute_multicall(calls, include_str!("../../abi/vault.json").to_string(), "tokenConfigurations").await.expect("[Vault] Failed to fetch token configurations");
        println!("data result {:?}", results);
        for (token, result) in tokens.iter_mut().zip(results) {
            println!("result {:?}", result);
            println!("result slice {:?}", result.as_slice());
            if let [is_whitelisted, _token_decimals, is_stable_token, is_shortable_token, min_profit_basis_points, token_weight, max_usdp_amount] = result.as_slice() {
                token.update_token_configuration(
                    token_weight.clone().into_uint().expect("Failed to parse token weight").as_u64(),
                    is_whitelisted.clone().into_bool().expect("Failed to parse is_whitelisted"),
                    is_stable_token.clone().into_bool().expect("Failed to parse is_stable_token"),
                    is_shortable_token.clone().into_bool().expect("Failed to parse is_shortable_token"),
                    min_profit_basis_points.clone().into_uint().expect("Failed to parse min_profit_basis_points").as_u64(),
                    max_usdp_amount.clone().into_uint().expect("Failed to parse max_usdp_amount"),
                );
            } else {
                anyhow::bail!("Invalid token configuration return data (may be invalid ABI), check vault.tokenConfigurations(address token) sm function");
            }
            
        }
        Ok(())
    }

    pub async fn fetch_token_prices(&self, tokens: &Mutex<Vec<Token>>) -> anyhow::Result<()> {
        let mut tokens = tokens.lock().await;
        // fetch ask price
        let fetch_ask_price_calls: Vec<(Address, Bytes)> = tokens.iter().map(|token| {
            let (vault_addr, data) = token.build_get_ask_price_call(&self.vault_addr);
            (vault_addr, data)
        }).collect();
        // fetch bid price
        let fetch_bid_price_calls: Vec<(Address, Bytes)> = tokens.iter().map(|token| {
            let (vault_addr, data) = token.build_get_bid_price_call(&self.vault_addr);
            (vault_addr, data)
        }).collect();
        let call_len = fetch_ask_price_calls.len();
        let mut merged_calls = fetch_bid_price_calls.clone();
        merged_calls.extend(fetch_ask_price_calls);
        let results = self.chain.execute_multicall(merged_calls, include_str!("../../abi/vault.json").to_string(), "getAskPrice").await.expect("[Vault] Failed to fetch ask prices");
        let chunk_reulsts = results.chunks(call_len);
        let ask_prices = chunk_reulsts.clone().next().expect("Failed to get ask prices")
            .into_iter().map(_format_price).collect::<Vec<_>>();
        let bid_prices = chunk_reulsts.clone().next().expect("Failed to get bid prices")
            .into_iter().map(_format_price).collect::<Vec<_>>();
        for (token, ask_price) in tokens.iter_mut().zip(ask_prices) {
            token.ask_price = Some(ask_price);
        }
        for (token, bid_price) in tokens.iter_mut().zip(bid_prices) {
            token.bid_price = Some(bid_price);
        }
        
        Ok(())
    }
}

fn _format_price(x: &Vec<ethabi::Token>) -> Decimal {
    return Decimal::from_str(&ethers::utils::format_units(x[0].clone().into_uint().expect("Failed to parse ask price"), 30).expect("failed to convert u256 to decimal")).expect("failed to convert price to decimal");
}


#[cfg(test)]
mod tests {
    use super::*;
    fn create_tokens() -> Vec<Token> {
        let mut tokens = vec![
            Token::new(97, "0x542E4676238562b518B968a1d03626d544a7BCA2", "USDT", "USDT", 18),
            Token::new(97, "0xc4900937c3222CA28Cd4b300Eb2575ee0868540F", "BTC", "BTC", 18),
        ];
        tokens
    }
    #[tokio::test]

    async fn test_fetch_token_configuration() {
        let vault = Vault {
            vault_addr: "0xF55Fc8e91c0c893568dB750cD4a4eB2D953E80a5".to_string(),
            chain: Chain {
                chain_id: 97,
                rpc_urls: vec!["https://data-seed-prebsc-1-s1.binance.org:8545/".to_string()],
                multicall_address: "0x6e5bb1a5ad6f68a8d7d6a5e47750ec15773d6042".to_string(),
            }
        };
        let tokens = Mutex::new(create_tokens());
        let result = vault.fetch_token_configuration(&tokens).await;
        assert!(result.is_ok());
        let tokens = tokens.lock().await;
        // verify tokens is modified
        assert_eq!(tokens[0].token_weight, Some(100));
        assert_eq!(tokens[0].is_stable_token, Some(true));
        assert_eq!(tokens[1].token_weight, Some(100));
        assert_eq!(tokens[1].is_stable_token, Some(false));
    }

    #[tokio::test]
    async fn test_fetch_token_prices() {
        let vault = Vault {
            vault_addr: "0xF55Fc8e91c0c893568dB750cD4a4eB2D953E80a5".to_string(),
            chain: Chain {
                chain_id: 97,
                rpc_urls: vec!["https://data-seed-prebsc-1-s1.binance.org:8545/".to_string()],
                multicall_address: "0x6e5bb1a5ad6f68a8d7d6a5e47750ec15773d6042".to_string(),
            }
        };

        let mut tokens = Mutex::new(create_tokens());
        let result = vault.fetch_token_prices(&tokens).await;
        assert!(result.is_ok());
        let tokens = tokens.lock().await;

        // verify token prices that > 0
        assert!(tokens[0].ask_price.unwrap().gt(&rust_decimal::Decimal::from(0)));
        assert!(tokens[1].ask_price.unwrap().gt(&rust_decimal::Decimal::from(0)));
        assert!(tokens[0].bid_price.unwrap().gt(&rust_decimal::Decimal::from(0)));
        assert!(tokens[1].bid_price.unwrap().gt(&rust_decimal::Decimal::from(0)));
    }
}
