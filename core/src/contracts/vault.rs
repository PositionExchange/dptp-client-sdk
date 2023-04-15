use std::{ops::Div, str::FromStr, time::Duration};

use ethers::types::{Address, Bytes, U256};
use serde::Serialize;
use tiny_keccak::{Keccak, Hasher};
use crate::config::Chain;
use super::token::{Token, Price};
use super::multicall::*;
use ethabi::Token as AbiToken;

#[derive(Default, Debug, Serialize, Clone, Copy)]
pub struct VaultState {
    pub usdp_address: Address,
    pub fee_basis_points: u32,
    pub tax_basis_points: u32,
    pub usdp_supply: U256,
    pub total_token_weights: U256,

    // min aum, max aum
    // get from plp manager
    pub total_aum: [U256; 2],
    pub plp_supply: U256,

    // vault state
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

#[derive(Default, Debug)]
pub struct Vault {
    pub vault_addr: String,
    pub plp_manager: String,
    pub plp_token: String,
    pub chain: Chain,
    pub state: VaultState,
}

impl Vault {
    pub fn new(vault_addr: &String, plp_manager: &String, plp_token: &String, chain: &Chain) -> Self {
        Self { vault_addr: vault_addr.to_string(), plp_token: plp_token.to_string(), plp_manager: plp_manager.to_string(), chain: chain.clone(), state: VaultState::default() }
    }

    pub async fn init_vault_state(&mut self) -> anyhow::Result<()> {
        // Note: Need to call init address first to initialize the addresses
        let _ = &self.init_address_state().await;
        // TODO move to join all?
        println!("addr {:?}", self.state.usdp_address);
        &self.init_vault_state_data().await;
        &self.init_plp_manager_state().await;
        Ok(())
    }

    /// Using multicall to fetch addresses from contracts
    async fn init_address_state(&mut self) -> anyhow::Result<()> {
        // Note This function is only to fetch addresses
        let calls = vec![
            get_encode_address_and_params(&self.vault_addr, &"usdp()".to_string(), &vec![]),
        ];
        let results = self.chain.execute_multicall_raw(calls).await.unwrap();
        let formated_results: Vec<_> = results.into_iter().map(
            |x| ethabi::decode(&[ethabi::ParamType::Address], &x).unwrap()
        ).collect();
        if let [usdp_addr] = &formated_results[..] {
            self.state.usdp_address = usdp_addr[0].clone().into_address().expect("Failed to parse usdp_addr");
            println!("init address state {:?}", self.state.usdp_address);
        } else {
            anyhow::bail!("Failed to parse addresses state, check your contract and ABI");
        }
        Ok(())
    }


    async fn init_plp_manager_state(&mut self) -> anyhow::Result<()> {
        let calls = vec![
            // get aum
            get_encode_address_and_params(&self.plp_manager, &"getAum(bool)".to_string(), &vec![AbiToken::Bool(true)]),
            get_encode_address_and_params(&self.plp_manager, &"getAum(bool)".to_string(), &vec![AbiToken::Bool(false)]),
            get_encode_address_and_params(&self.plp_token, &"totalSupply()".to_string(), &vec![]),
            (self.state.usdp_address, encode_selector_and_params(&"totalSupply()".to_string(), &vec![])),
        ];
        let results = self.chain.execute_multicall_raw(calls).await.unwrap();
        println!("usdp_address {:?}", self.state.usdp_address);

        let formated_results: Vec<_> = results.into_iter().map(
            |x| ethabi::decode(&[ethabi::ParamType::Uint(256)], &x).unwrap()
        ).collect();
        if let [aum1, aum2, plp_supply, usdp_supply] = &formated_results[..] {
            self.state.total_aum[0] = aum1[0].clone().into_uint().expect("Failed to parse aum1");
            self.state.total_aum[1] = aum2[0].clone().into_uint().expect("Failed to parse aum2");
            self.state.plp_supply = plp_supply[0].clone().into_uint().expect("Failed to parse plp_supply");
            self.state.usdp_supply = usdp_supply[0].clone().into_uint().expect("Failed to parse usdp_supply");
        } else {
            anyhow::bail!("Failed to fetch plp manager state. Maybe invalid contract ABI");
        }

        Ok(())
    }

    async fn init_vault_state_data(&mut self) -> anyhow::Result<()> {
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
            get_vault_variable_selector(&self.vault_addr.clone(), &"totalTokenWeight".to_string()),
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
        stable_borrowing_rate_factor,
        total_token_weights
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
            self.state.total_token_weights = total_token_weights[0].clone().into_uint().expect("Failed to parse total_token_weights");
        } else {
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

    pub async fn fetch_vault_info(&self, tokens: &mut Vec<Token>) -> anyhow::Result<()> {

        let calls: Vec<(Address, Bytes)> = tokens.iter().map(|token| {
            let (vault_addr, data) = token.build_get_vault_info(&self.vault_addr);
            (vault_addr, data)
        }).collect();


        let results = self.chain.execute_multicall(calls, include_str!("../../abi/vault.json").to_string(), "vaultInfo").await.expect("[Vault] Failed to fetch vault info");
        println!("data result {:?}", results);

        for (token, result) in tokens.iter_mut().zip(results) {
            println!("result {:?}", result);
            println!("result slice {:?}", result.as_slice());
            if let [
                feeReserves,
                usdpAmounts,
                poolAmounts,
                reservedAmounts] = result.as_slice() {
                token.update_vault_info(
                    usdpAmounts.clone().into_uint().expect("Fail to parse usdp amount"),
                    feeReserves.clone().into_uint().expect("Fail to parse feeReserves"),
                    poolAmounts.clone().into_uint().expect("Fail to parse poolAmounts"),
                    reservedAmounts.clone().into_uint().expect("Fail to parse reservedAmounts"),
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

        let fetch_ask_price_calls: Vec<(Address, Bytes)> = tokens.iter()
            .filter(|token| token.is_tradeable == Some(true))
            .map(|token| {
            let (vault_addr, data) = token.build_get_ask_price_call(&self.vault_addr);
            (vault_addr, data)
        }).collect();

        // fetch bid price
        let fetch_bid_price_calls: Vec<(Address, Bytes)> = tokens.iter()
            .filter(|token| token.is_tradeable == Some(true))
            .map(|token| {
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

        println!("**********done fetch_token_prices********");


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

fn get_encode_address_and_params(address: &str, function_signature: &str, params: &[ethabi::Token]) -> (Address, Bytes) {
    let address = Address::from_str(address).expect("Failed to parse address");
    let data = encode_selector_and_params(function_signature, params);
    (address, data)
}

fn encode_selector_and_params(function_signature: &str, params: &[ethabi::Token]) -> Bytes {
    let selector = _get_function_selector(function_signature);
    let mut encoded = selector.to_vec();
    encoded.extend(ethabi::encode(params));
    Bytes::from(encoded)
}


#[cfg(test)]
mod tests {
    use ethers::utils::hex;
    use crate::contracts::vault_logic::VaultLogic;

    use super::*;

    fn create_tokens() -> Vec<Token> {
        let mut tokens = vec![
            Token::new(97, "0x542E4676238562b518B968a1d03626d544a7BCA2", "USDT", "USDT", 18, ""),
            Token::new(97, "0xc4900937c3222CA28Cd4b300Eb2575ee0868540F", "BTC", "BTC", 18, ""),
        ];
        tokens
    }

    fn create_vault() -> Vault {
        let chain = Chain {
            chain_id: 97,
            rpc_urls: vec!["https://data-seed-prebsc-1-s1.binance.org:8545/".to_string()],
            multicall_address: "0x6e5bb1a5ad6f68a8d7d6a5e47750ec15773d6042".to_string(),
        };
        return Vault::new(&"0x3e6fb757447d34347AD940E0E789d976a1cf3842".to_string(), &"0xDF49C2d458892B681331F4EEC0d09A88b283f444".to_string(), &"0x792bA5e9E0Cd15083Ec2f58E434d875892005b91".to_string(), &chain);
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
        let vault = create_vault();
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
    async fn test_fetch_vault_info() {
        let vault = create_vault();
        let mut tokens = create_tokens();
        println!("len {} ", tokens.len());

        let result = vault.fetch_token_configuration(&mut tokens).await;
        let result = vault.fetch_vault_info(&mut tokens).await;
        assert_eq!(tokens[0].usdp_amount, Some(U256::zero()));
        assert_eq!(tokens[0].reserved_amounts, Some(U256::zero()));
        assert_eq!(tokens[0].pool_amounts, Some(U256::zero()));
        assert_eq!(tokens[0].fee_reserves, Some(U256::zero()));


        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fetech_vault_state() {
        let mut vault = create_vault();
        // let tokens = Mutex::new(create_tokens());
        let result = vault.init_vault_state().await;
        assert!(result.is_ok());
        // let tokens = tokens.lock().await;
        // expect vault state is modified
        // expect mint burn fee < 0
        assert!(vault.state.mint_burn_fee_basis_points > U256::from(0));
        assert!(vault.state.swap_fee_basis_points > U256::from(0));
        assert!(vault.state.stable_tax_basis_points > U256::from(0));
        assert!(vault.state.fee_basis_points >= 0);
        assert!(vault.state.tax_basis_points >= 0);
        assert!(vault.state.usdp_supply > U256::zero());
        assert!(vault.state.total_token_weights > U256::zero());
        assert!(vault.state.total_aum[0] > U256::zero());
        assert!(vault.state.total_aum[1] > U256::zero());
        assert!(vault.state.plp_supply > U256::zero());
        assert!(vault.state.mint_burn_fee_basis_points > U256::zero());
        assert!(vault.state.swap_fee_basis_points > U256::zero());
        assert!(vault.state.stable_swap_fee_basis_points > U256::zero());
        assert!(vault.state.margin_fee_basis_points > U256::zero());
        assert!(vault.state.stable_tax_basis_points > U256::zero());
        assert!(vault.state.liquidation_fee_usd >= U256::zero());
        assert!(vault.state.borrowing_rate_factor > U256::zero());
        assert!(vault.state.stable_borrowing_rate_factor > U256::zero());
    }


    #[tokio::test]
    async fn test_fetch_token_prices() {
        let vault = create_vault();

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

    #[tokio::test]
    async fn test_init_plp_manager_state() {
        let mut vault = create_vault();
        let result = vault.init_plp_manager_state().await;

        assert_eq!(result.is_ok(), true);

        assert!(vault.state.total_aum[0] > U256::from(0));
        assert!(vault.state.total_aum[1] > U256::from(0));
        assert!(vault.state.plp_supply > U256::from(0));
    }

    #[test]
    fn test_encode_selector_and_params() {
        let function_signature = "transfer(address,uint256)";
        let params = vec![ethabi::Token::Address(Address::from_str("0x6e5bb1a5ad6f68a8d7d6a5e47750ec15773d6042").unwrap()), ethabi::Token::Uint(U256::from(100))];
        let result = encode_selector_and_params(function_signature, &params);
        println!("{}", hex::encode(result.clone()));
        assert_eq!(hex::encode(result.clone()), "a9059cbb0000000000000000000000006e5bb1a5ad6f68a8d7d6a5e47750ec15773d60420000000000000000000000000000000000000000000000000000000000000064");
    }

    #[tokio::test]
    async fn test_buy_plp_to_amount(){
        let vault = create_vault();
        let mut tokens = create_tokens();
        println!("len {} ", tokens.len());

        let result = vault.fetch_token_configuration(&mut tokens).await;
        let result = vault.fetch_vault_info(&mut tokens).await;


        let (result, fee) = vault.state.get_buy_glp_to_amount(&U256::from_dec_str("1000000000000000000").unwrap(), &tokens[0] );

        println!("result {}", result);

    }
}
