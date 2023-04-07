use ethers::{
    types::{Address, Bytes, U256},
};

use crate::config::Chain;

use super::token::Token;
use super::multicall::*;



pub struct Vault {
    vault_addr: String,
    chain: Chain,
}

impl Vault {
    pub async fn fetch_token_configuration(&self, tokens: &mut Vec<Token>) -> anyhow::Result<()> {
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
}


#[cfg(test)]
mod tests {
    use super::*;
    fn create_tokens() -> Vec<Token> {
        let mut tokens = vec![
            Token {
                    chain_id: Some(97),
                    address: "0x542E4676238562b518B968a1d03626d544a7BCA2".to_string(),
                    name: "USDT".to_string(),
                    symbol: "USDT".to_string(),
                    decimals: 18,
                    token_weight: None,
                    is_whitelisted: None,
                    is_stable_token: None,
                    is_shortable_token: None,
                    min_profit_basis_points: None,
                    max_usdp_amount: None,
                    is_native_token: None,
                    allowances: None,
                    balances: None,
                },
            Token {
                    chain_id: Some(97),
                    address: "0xc4900937c3222CA28Cd4b300Eb2575ee0868540F".to_string(),
                    name: "BTC".to_string(),
                    symbol: "BTC".to_string(),
                    decimals: 18,
                    token_weight: None,
                    is_whitelisted: None,
                    is_stable_token: None,
                    is_shortable_token: None,
                    min_profit_basis_points: None,
                    max_usdp_amount: None,
                    is_native_token: None,
                    allowances: None,
                    balances: None,
                },
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
        let mut tokens = create_tokens();
        let result = vault.fetch_token_configuration(&mut tokens).await;
        assert!(result.is_ok());
        // verify tokens is modified
        assert_eq!(tokens[0].token_weight, Some(100));
        assert_eq!(tokens[0].is_stable_token, Some(true));
        assert_eq!(tokens[1].token_weight, Some(100));
        assert_eq!(tokens[1].is_stable_token, Some(false));
    }
}
