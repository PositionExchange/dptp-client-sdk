use std::collections::HashMap;
use ethers::{types::Bytes};
use ethabi::{ethereum_types::Address, ethereum_types::U256, Contract};
use std::str::FromStr;
use serde::Deserialize;
use rust_decimal::prelude::Decimal;

#[derive(Debug, Deserialize, Clone)]
pub struct Token {
    pub chain_id: Option<u64>,
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    // get from Vault.tokenConfigurations(address token) function
    pub token_weight: Option<u64>,
    pub is_whitelisted: Option<bool>,
    pub is_stable_token: Option<bool>,
    pub is_shortable_token: Option<bool>,
    pub min_profit_basis_points: Option<u64>,
    pub max_usdp_amount: Option<U256>,

    pub is_native_token: Option<bool>,
    pub allowances: Option<HashMap<Address, U256>>,
    pub balances: Option<HashMap<Address, U256>>,
}

impl Token {
    pub fn build_balance_of_call(&self, account: &String) -> (Address, Bytes) {
        let address: Address = account.parse().unwrap();
        let token: Address = self.address.parse().unwrap();
        let function_name = "balanceOf";
        let erc20_abi = include_str!("../../abi/erc20.json");
        let contract = Contract::load(erc20_abi.as_bytes()).unwrap();
        let data: Bytes = contract.function(function_name).unwrap().encode_input(&[ethabi::Token::Address(address)]).unwrap().into();
        (token, data)
    }

    pub fn build_allowance_of_call(&self, account: &String) -> (Address, Bytes) {
        let address: Address = account.parse().unwrap();
        let token: Address = self.address.parse().unwrap();
        let function_name = "allowance";
        let erc20_abi = include_str!("../../abi/erc20.json");
        let contract = Contract::load(erc20_abi.as_bytes()).unwrap();
        let function = contract.function(function_name).unwrap();
        let data: Bytes = function.encode_input(&[
            ethabi::Token::Address(address),
            ethabi::Token::Address(address),
        ])
        .unwrap().into();
        (token, data)
    }

    pub fn build_get_vault_token_configuration_call(&self, vault_address: &String) -> (Address, Bytes) {
        let address: Address = vault_address.parse().unwrap();
        let function_name = "tokenConfigurations";
        let contract_abi = include_str!("../../abi/vault.json");
        let contract = Contract::load(contract_abi.as_bytes()).unwrap();
        let function = contract.function(function_name).unwrap();
        let data: Bytes = function.encode_input(&[ethabi::Token::Address(self.get_parsed_address())]).unwrap().into();
        (address, data)
    }


    pub fn update_token_configuration(
        &mut self,
        token_weight: u64, 
        is_whitelisted: bool,
        is_stable_token: bool,
        is_shortable_token: bool,
        min_profit_basis_points: u64,
        max_usdp_amount: U256,
    ) {
        self.token_weight = Some(token_weight);
        self.is_whitelisted = Some(is_whitelisted);
        self.is_stable_token = Some(is_stable_token);
        self.is_shortable_token = Some(is_shortable_token);
        self.min_profit_basis_points = Some(min_profit_basis_points);
        self.max_usdp_amount = Some(max_usdp_amount);
    }

    pub fn update_balance(&mut self, account: &String, balance: U256) {
        let addr: Address = account.parse().unwrap();
        if self.balances.is_none() {
            self.balances = Some(HashMap::new());
        }
        self.balances.as_mut().unwrap().insert(addr, balance);
    }

    pub fn update_allowance(&mut self, account: &String, allowance: U256) {
        let addr: Address = account.parse().unwrap();
        if self.allowances.is_none() {
            self.allowances = Some(HashMap::new());
        }
        self.allowances.as_mut().unwrap().insert(addr, allowance);
    }

    pub fn get_balance(&self, account: &String) -> String {
        let addr: Address = account.parse().unwrap();
        let binding = self.balances
            .clone()
            .unwrap_or_else(|| HashMap::new());
        let balance = binding
            .get(&addr);
        let zero = U256::from(0);
        return balance.unwrap_or_else(|| &zero).to_string();
    }
    pub fn get_allowance(&self, account: &String) -> String {
        let addr: Address = account.parse().unwrap();
        let binding = self.allowances
            .clone()
            .unwrap_or_else(|| HashMap::new());
        let val = binding
            .get(&addr);
        let zero = U256::from(0);
        return val.unwrap_or_else(|| &&zero).to_string();
    }

    pub fn get_token_ratio(&self, total_weight: &u64) -> Decimal {
        let weight = self.token_weight.unwrap_or(0);
        let ratio = Decimal::from(weight) / Decimal::from(*total_weight);
        ratio
    }

    fn get_parsed_address(&self) -> Address {
        return self.address.parse().expect("Token address parse error");
    }
}


#[cfg(test)]
mod tests {
    use ethers::utils::hex;
    use super::*;

    fn create_mock_token() -> Token {
         let token = Token {
            chain_id: Some(97),
            address: "0x1f9840a85d5af5bf1d1762f925bdaddc4201f984".to_string(),
            name: "Uniswap".to_string(),
            symbol: "UNI".to_string(),
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
        };
        token
    }

    #[test]
    fn build_balance_of_call_works() {
        let token = create_mock_token();
        let (address, data) = token.build_balance_of_call(&"0x1f9840a85d5af5bf1d1762f925bdaddc4201f984".to_string());
        let data_string = hex::encode(data.clone());
        println!("[balance call] data {}", data_string);
        assert_eq!(address, Address::from_str("0x1f9840a85d5af5bf1d1762f925bdaddc4201f984").unwrap());
        assert_eq!(data_string, "70a082310000000000000000000000001f9840a85d5af5bf1d1762f925bdaddc4201f984".to_string());
    }

    #[test]
    fn build_allowance_of_call_works() {
        let token = create_mock_token();
        let (address, data) = token.build_allowance_of_call(&"0x1f9840a85d5af5bf1d1762f925bdaddc4201f984".to_string());
        let data_string = hex::encode(data.clone());
        assert_eq!(address, Address::from_str("0x1f9840a85d5af5bf1d1762f925bdaddc4201f984").unwrap());
        assert_eq!(data_string, "dd62ed3e0000000000000000000000001f9840a85d5af5bf1d1762f925bdaddc4201f9840000000000000000000000001f9840a85d5af5bf1d1762f925bdaddc4201f984".to_string());
    }

    #[test]
    fn build_get_vault_token_configuration_call_works() {
        let token = create_mock_token();
        let vault_address_mock = "0x1f9840a85d5af5bf1d1762f925bdaddc4201f986".to_string();
        let (address, data) = token.build_get_vault_token_configuration_call(&vault_address_mock);
        let data_string = hex::encode(data.clone());
        assert_eq!(address, Address::from_str(&vault_address_mock).unwrap());
        assert_eq!(data_string, "9b2ac49a0000000000000000000000001f9840a85d5af5bf1d1762f925bdaddc4201f984".to_string());
    }


    // test for update functions
    #[test]
    fn update_token_configuration_works() {
        let mut token = create_mock_token();
        token.update_token_configuration(1000, true, true, true, 500, U256::from(1000));
        assert_eq!(token.token_weight.unwrap(), 1000);
        assert_eq!(token.is_whitelisted.unwrap(), true);
        assert_eq!(token.is_stable_token.unwrap(), true);
        assert_eq!(token.is_shortable_token.unwrap(), true);
        assert_eq!(token.min_profit_basis_points.unwrap(), 500);
        assert_eq!(token.max_usdp_amount.unwrap(), U256::from(1000));
    }

    #[test]
    fn update_balance_works() {
        let mut token = create_mock_token();
        let user1 = "0x1f9840a85d5af5bf1d1762f925bdaddc4201f984".to_string();
        let user2 = "0x1f9840a85d5af5bf1d1762f925bdaddc4201f985".to_string();
        let user3 = "0x1f9840a85d5af5bf1d1762f925bdaddc4201f986".to_string();
        token.update_balance(&user1, U256::from(1000));
        assert_eq!(token.get_balance(&user1), "1000");
        // for multiple users shouild work
        token.update_balance(&user2, U256::from(2000));
        assert_eq!(token.get_balance(&user1), "1000");
        assert_eq!(token.get_balance(&user2), "2000");
        assert_eq!(token.get_balance(&user3), "0");
    }

    #[test]
    fn update_allowance_works() {
        let mut token = create_mock_token();
        let user1 = "0x1f9840a85d5af5bf1d1762f925bdaddc4201f984".to_string();
        let user2 = "0x1f9840a85d5af5bf1d1762f925bdaddc4201f985".to_string();
        let user3 = "0x1f9840a85d5af5bf1d1762f925bdaddc4201f986".to_string();
        token.update_allowance(&user1, U256::from(1000));
        assert_eq!(token.get_allowance(&user1), "1000");
        token.update_allowance(&user2, U256::from(5000));
        assert_eq!(token.get_allowance(&user1), "1000");
        assert_eq!(token.get_allowance(&user2), "5000");
        token.update_allowance(&user2, U256::from(6000));
        assert_eq!(token.get_allowance(&user2), "6000");
        assert_eq!(token.get_allowance(&user3), "0");
    }

    #[test]
    fn calculate_ratio_correctly() {
        let mut token = create_mock_token();
        token.update_token_configuration(1000, true, true, true, 500, U256::from(1000));
        assert!(token.get_token_ratio(&1000).eq(&Decimal::from(1)));
        assert!(token.get_token_ratio(&2000).eq(&rust_decimal_macros::dec!(0.5)));
        assert!(token.get_token_ratio(&300).eq(&rust_decimal_macros::dec!(3.3333333333333333333333333333)));
    }

}

