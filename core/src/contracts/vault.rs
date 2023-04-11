use std::{ops::Div, str::FromStr, time::Duration};

use ethers::{
    types::{Address, Bytes, U256},
};
use rust_decimal::Decimal;
use tiny_keccak::{Keccak, Hasher};

use crate::config::Chain;

use super::token::{Token, Price};
use super::multicall::*;

#[derive(Default)]
pub struct VaultState {
    pub fee_basis_points: u32,
    pub tax_basis_points: u32,
    pub usdp_supply: U256,
    pub total_token_weights: U256,

    pub mint_burn_fee_basis_points: U256,
    pub swap_fee_basis_points: U256,
    pub stable_swap_fee_basis_points: U256,
    pub margin_fee_basis_points: U256,
    pub stable_tax_basis_points: U256,
    pub has_dynamic_fees: bool,
    pub in_manager_mode: bool,
    pub is_swap_enabled: bool,
    pub liquidation_fee_usd: U256,
    pub borrowing_rate_interval: Duration,
    pub borrowing_rate_factor: U256,
    pub stable_borrowing_rate_factor: U256,
}

#[derive(Default)]
pub struct Vault {
    vault_addr: String,
    chain: Chain,
    state: VaultState,
}

impl Vault {
    pub fn new(vault_addr: &String, chain: &Chain) -> Self {
        Self { vault_addr: vault_addr.to_string(), chain: chain.clone() , state: VaultState::default() }
    }

    pub async fn fetch_vault_state(&mut self) -> anyhow::Result<()> {
        let calls = vec![
            get_vault_variable_selector(&self.vault_addr.clone(), &"mintBurnFeeBasisPoints".to_string()),
            get_vault_variable_selector(&self.vault_addr.clone(), &"swapFeeBasisPoints".to_string()),
            get_vault_variable_selector(&self.vault_addr.clone(), &"stableSwapFeeBasisPoints".to_string()),
            get_vault_variable_selector(&self.vault_addr.clone(), &"marginFeeBasisPoints".to_string()),
            get_vault_variable_selector(&self.vault_addr.clone(), &"taxBasisPoints".to_string()),
            get_vault_variable_selector(&self.vault_addr.clone(), &"stableTaxBasisPoints".to_string()),
            get_vault_variable_selector(&self.vault_addr.clone(), &"hasDynamicFees".to_string()),
            get_vault_variable_selector(&self.vault_addr.clone(), &"inManagerMode".to_string()),
            get_vault_variable_selector(&self.vault_addr.clone(), &"isSwapEnabled".to_string()),
            get_vault_variable_selector(&self.vault_addr.clone(), &"liquidationFeeUsd".to_string()),
            get_vault_variable_selector(&self.vault_addr.clone(), &"borrowingRateInterval".to_string()),
            get_vault_variable_selector(&self.vault_addr.clone(), &"borrowingRateFactor".to_string()),
            get_vault_variable_selector(&self.vault_addr.clone(), &"stableBorrowingRateFactor".to_string()),
        ];
        let results = self.chain.execute_multicall(calls, include_str!("../../abi/vault.json").to_string(), "mintBurnFeeBasisPoints").await.expect("Failed to fetch vault state");
        if let [
            mint_burn_fee_basis_points,
            swap_fee_basis_points,
            stable_swap_fee_basis_points,
            margin_fee_basis_points,
            tax_basis_points,
            stable_tax_basis_points,
            has_dynamic_fees,
            in_manager_mode,
            is_swap_enabled,
            liquidation_fee_usd,
            borrowing_rate_interval,
            borrowing_rate_factor,
            stable_borrowing_rate_factor
        ] = results.as_slice() {
            self.state.mint_burn_fee_basis_points = mint_burn_fee_basis_points[0].clone().into_uint().expect("Failed to parse mint_burn_fee_basis_points");
            self.state.swap_fee_basis_points = swap_fee_basis_points[0].clone().into_uint().expect("Failed to parse swap_fee_basis_points");
            self.state.stable_swap_fee_basis_points = stable_swap_fee_basis_points[0].clone().into_uint().expect("Failed to parse stable_swap_fee_basis_points");
            self.state.margin_fee_basis_points = margin_fee_basis_points[0].clone().into_uint().expect("Failed to parse margin_fee_basis_points");
            self.state.tax_basis_points = tax_basis_points[0].clone().into_uint().expect("Failed to parse tax_basis_points").as_u32();
            self.state.stable_tax_basis_points = stable_tax_basis_points[0].clone().into_uint().expect("Failed to parse stable_tax_basis_points");
            self.state.has_dynamic_fees = if has_dynamic_fees[0].clone().into_uint().expect("Failed to parse has_dynamic_fees").as_u32() == 1 { true } else { false };
            self.state.in_manager_mode = if in_manager_mode[0].clone().into_uint().expect("Failed to parse in_manager_mode").as_u32() == 1 { true } else { false };
            self.state.is_swap_enabled = if is_swap_enabled[0].clone().into_uint().expect("Failed to parse is_swap_enabled").as_u32() == 1 { true } else { false };
            self.state.liquidation_fee_usd = liquidation_fee_usd[0].clone().into_uint().expect("Failed to parse liquidation_fee_usd");
            self.state.borrowing_rate_interval = Duration::from_secs(borrowing_rate_interval[0].clone().into_uint().expect("Failed to parse borrowing_rate_interval").as_u64());
            self.state.borrowing_rate_factor = borrowing_rate_factor[0].clone().into_uint().expect("Failed to parse borrowing_rate_factor");
            self.state.stable_borrowing_rate_factor = stable_borrowing_rate_factor[0].clone().into_uint().expect("Failed to parse stable_borrowing_rate_factor");
        }else{
            anyhow::bail!("Invalid vault state return data (may be invalid ABI), check Vault smart contract");
        }
        println!("call results {:?}", results);
        Ok(())
    }

    pub async fn fetch_token_configuration(&self, tokens: &mut Vec<Token>) -> anyhow::Result<()> {
         // let mut tokens = tokens.lock().await;
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

    pub async fn fetch_token_prices(&self, tokens: &mut Vec<Token>) -> anyhow::Result<()> {
        // let mut tokens = tokens.lock().await;
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

fn _format_price(x: &Vec<ethabi::Token>) -> Price {
    return Price::new_from_eth_token(&x[0]);
}

fn get_vault_variable_selector(vault_address: &str, variable_name: &str) -> (Address, Bytes) {
    let address = Address::from_str(vault_address).expect("Failed to parse vault address");
    let fn_selector_raw = _get_function_selector(&format!("{}()", variable_name));
    (address, Bytes::from(fn_selector_raw))
}

fn _get_function_selector(function_signature: &str) -> [u8; 4] {
    let mut keccak = Keccak::v256();
    let mut output = [0u8; 32];
    let mut selector = [0u8; 4];

    keccak.update(function_signature.as_bytes());
    keccak.finalize(&mut output);

    selector.copy_from_slice(&output[0..4]);
    selector
}



#[cfg(test)]
mod tests {
    use ethers::utils::hex;

    use super::*;
    fn create_tokens() -> Vec<Token> {
        let mut tokens = vec![
            Token::new(97, "0x542E4676238562b518B968a1d03626d544a7BCA2", "USDT", "USDT", 18),
            Token::new(97, "0xc4900937c3222CA28Cd4b300Eb2575ee0868540F", "BTC", "BTC", 18),
        ];
        tokens
    }
    #[test]
    fn test_get_function_selector() {
        let function_signature = "transfer(address,uint256)";
        let expected_selector = [0xa9, 0x05, 0x9c, 0xbb]; // This is the expected selector for the "transfer(address,uint256)" function
        let selector = _get_function_selector(function_signature);

        assert_eq!(selector, expected_selector, "Unexpected function selector");
        assert_eq!(hex::encode(selector), "0xa9059cbb", "Unexpected function selector");
    }
    #[tokio::test]
    async fn test_fetch_token_configuration() {
        let vault = Vault {
            vault_addr: "0xF55Fc8e91c0c893568dB750cD4a4eB2D953E80a5".to_string(),
            chain: Chain {
                chain_id: 97,
                rpc_urls: vec!["https://data-seed-prebsc-1-s1.binance.org:8545/".to_string()],
                multicall_address: "0x6e5bb1a5ad6f68a8d7d6a5e47750ec15773d6042".to_string(),
            },
            state: VaultState::default(),
        };
        // let tokens = Mutex::new(create_tokens());
        let mut tokens = create_tokens();
        let result = vault.fetch_token_configuration(&mut tokens).await;
        assert!(result.is_ok());
        // let tokens = tokens.lock().await;
        // verify tokens is modified
        assert_eq!(tokens[0].token_weight, Some(100));
        assert_eq!(tokens[0].is_stable_token, Some(true));
        assert_eq!(tokens[1].token_weight, Some(100));
        assert_eq!(tokens[1].is_stable_token, Some(false));
    }

    #[tokio::test]
    async fn test_fetech_vault_state() {
        let mut vault = Vault {
            vault_addr: "0x3e6fb757447d34347AD940E0E789d976a1cf3842".to_string(),
            chain: Chain {
                chain_id: 97,
                rpc_urls: vec!["https://data-seed-prebsc-1-s1.binance.org:8545/".to_string()],
                multicall_address: "0x6e5bb1a5ad6f68a8d7d6a5e47750ec15773d6042".to_string(),
            },
            state: VaultState::default(),
        };
        // let tokens = Mutex::new(create_tokens());
        let result = vault.fetch_vault_state().await;
        assert!(result.is_ok());
        // let tokens = tokens.lock().await;
        // expect vault state is modified
        // expect mint burn fee < 0
        assert!(vault.state.mint_burn_fee_basis_points > U256::from(0));
        assert!(vault.state.swap_fee_basis_points > U256::from(0));
        assert!(vault.state.stable_tax_basis_points > U256::from(0));
    }


    #[tokio::test]
    async fn test_fetch_token_prices() {
        let vault = Vault {
            vault_addr: "0xF55Fc8e91c0c893568dB750cD4a4eB2D953E80a5".to_string(),
            chain: Chain {
                chain_id: 97,
                rpc_urls: vec!["https://data-seed-prebsc-1-s1.binance.org:8545/".to_string()],
                multicall_address: "0x6e5bb1a5ad6f68a8d7d6a5e47750ec15773d6042".to_string(),
            },
            state: VaultState::default(),
        };

        // let mut tokens = Mutex::new(create_tokens());
        let mut tokens = create_tokens();
        let result = vault.fetch_token_prices(&mut tokens).await;
        assert!(result.is_ok());
        // let tokens = tokens.lock().await;

        // verify token prices that > 0
        assert!(tokens[0].ask_price.unwrap().parsed.gt(&rust_decimal::Decimal::from(0)));
        assert!(tokens[1].ask_price.unwrap().parsed.gt(&rust_decimal::Decimal::from(0)));
        assert!(tokens[0].bid_price.unwrap().parsed.gt(&rust_decimal::Decimal::from(0)));
        assert!(tokens[1].bid_price.unwrap().parsed.gt(&rust_decimal::Decimal::from(0)));
    }
}
