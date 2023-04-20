use ethers::{
    abi::{Abi, Token, Function},
    prelude::{abigen, Abigen},
    providers::{Http, Provider},
    types::{Address, Bytes, U256},
};
use std::sync::Arc;
use crate::config::Chain;
use async_trait::async_trait;
use rand::Rng;

abigen!(
    Multicall,
    "./abi/multicall.json",
);

#[async_trait(?Send)]
pub trait ChainMulticallTrait {
    //! execute multicall
    //! pass calls to multicall contract
    //! pass the interface and function name to decode the return data
    async fn execute_multicall(&self, calls: Vec<(Address, Bytes)>, interface: String, fn_name: &str) -> Result<Vec<Vec<Token>>, String>;
    async fn execute_multicall_raw(&self, calls: Vec<(Address, Bytes)>) -> Result<Vec<Bytes>, String>;
}

#[async_trait(?Send)]
impl ChainMulticallTrait for Chain {
    async fn execute_multicall(&self, calls: Vec<(Address, Bytes)>, interface: String, fn_name: &str) -> Result<Vec<Vec<Token>>, String>{
        let return_data = self.execute_multicall_raw(calls).await;
        // convert return data to type
        Ok(decode_return_data(return_data.unwrap(), interface.clone(), fn_name))
    }
    async fn execute_multicall_raw(&self, calls: Vec<(Address, Bytes)>) -> Result<Vec<Bytes>, String>{
        let mut rng = rand::thread_rng();
        let random_index = rng.gen_range(0..4);

        println!("random_index {}", random_index);

        let provider = Provider::<Http>::try_from(self.rpc_urls[random_index].clone()).expect("invalid rpc url, check your config");
        let client = Arc::new(provider.clone());
        let address: Address = self.multicall_address.parse().expect("invalid multicall address, check your config");
        let multicall = Multicall::new(address, client);
        let (_block_number, return_data) = multicall.aggregate(calls).call().await.expect("Failed to execute multicall");
        Ok(return_data)
    }
}

// decode return data by interface
fn decode_return_data(return_data: Vec<Bytes>, interface: String, fn_name: &str) -> Vec<Vec<ethabi::Token>> {
    let abi = ethabi::Contract::load(interface.as_bytes()).unwrap();
    let function = abi.function(fn_name).unwrap();
    let results: Vec<Vec<ethabi::Token>> = return_data.into_iter().map(|data| function.decode_output(&data).unwrap()).collect();
    results
}


#[cfg(test)]
mod tests {
    use super::*;
    use ethers::types::U256;

    #[tokio::test]
    async fn execute_multicall_works() {
        let chain = Chain {
            chain_id: 97,
            rpc_urls: vec!["https://data-seed-prebsc-1-s1.binance.org:8545".to_string()],
            multicall_address: "0x6e5bb1a5ad6f68a8d7d6a5e47750ec15773d6042".to_string(),
        };
        let calls = vec![
            (
                "0xFa60D973F7642B748046464e165A65B7323b0DEE".parse().unwrap(),
                "0x70a0823100000000000000000000000040682a04d9aa11c0bcdc7fa503c409fcf0a2e02e".parse().unwrap(),
            ),
            (
                "0xFa60D973F7642B748046464e165A65B7323b0DEE".parse().unwrap(),
                "0x70a08231000000000000000000000000d7b71d0e8a1e6b7c0b8f9c7c3d3d1d7f1b1d7b71".parse().unwrap(),
            ),
        ];
        let erc20_abi = r#"
           [
                {
                    "constant": true,
                    "inputs": [
                        {
                            "name": "_owner",
                            "type": "address"
                        }
                    ],
                    "name": "balanceOf",
                    "outputs": [
                        {
                            "name": "balance",
                            "type": "uint256"
                        }
                    ],
                    "payable": false,
                    "stateMutability": "view",
                    "type": "function"
                }
           ] 
        "#;

        let data = chain.execute_multicall(calls, erc20_abi.to_string(), "balanceOf").await.unwrap();
        println!("data: {:?}", data);
        // decode data
        assert!(data[0][0].clone().into_uint().unwrap().gt(&U256::zero()));
        assert!(data[1][0].clone().into_uint().unwrap().eq(&U256::zero()));
    }
}

