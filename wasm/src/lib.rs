use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::{to_value, from_value};
use serde::{Serialize, Deserialize};
use core::{*};
use std::sync::{Arc, Mutex};
use console_error_panic_hook;
use ethabi::ethereum_types::U256;
use core::contracts::vault_logic::VaultLogic;
use core::contracts::token::Token;

#[wasm_bindgen]
pub struct WasmRouter {
    router: core::Router,
}

// Note: this is the example to custom export struct to js
// #[wasm_bindgen]
// pub struct WasmChain {
//     pub chain_id: u64,
//     rpc_urls: js_sys::Array,
//     multicall_address: JsValue,
// }
//
// impl From<core::config::Chain> for WasmChain {
//     fn from(chain: core::config::Chain) -> Self {
//         let rpc_urls: js_sys::Array = chain
//             .rpc_urls
//             .into_iter()
//             .map(|x| JsValue::from_str(&x))
//             .collect();
//
//         WasmChain {
//             chain_id: chain.chain_id,
//             rpc_urls: rpc_urls.into(),
//             multicall_address: chain.multicall_address.into(),
//         }
//     }
//
// }
// #[wasm_bindgen]
// impl WasmChain {
//     #[wasm_bindgen(getter)]
//     pub fn rpc_urls(&self) -> js_sys::Array {
//         self.rpc_urls.clone()
//     }
// }

trait WasmRouterTrait {}

#[wasm_bindgen]
impl WasmRouter {
    #[wasm_bindgen(constructor)]
    pub fn new(chain_id: u64) -> Self {
        let mut router = Router::new();


        router.initilize(chain_id).expect("load_config failed");

        WasmRouter {
            router,
        }
    }

    pub fn load_config(&mut self, chain_id: u64) -> Result<JsValue, JsValue> {
        match self.router.initilize(chain_id) {
            Ok(config) => Ok(to_value(config).unwrap()),
            Err(e) => Err(JsValue::from_str(e)),
        }
    }

    pub fn load_tokens(&self) -> JsValue {
        let tokens = self.router.load_tokens();
        to_value(&tokens).unwrap()
    }
    //
    // pub fn load_tokens_string(&self) -> String {
    //     let tokens = self.router.load_tokens();
    //
    //     return serde_json::to_string(&tokens).unwrap()
    //     // to_value(&tokens).unwrap()
    // }

    pub fn set_account(&mut self, account: String) {
        self.router.set_account(account);
    }


    pub async fn fetch_data(&mut self) -> Result<(), JsValue> {
        self.router.fetch_data().await.map_err(|e| JsValue::from_str(&e.to_string())).expect("featch data failure");
        Ok(())
    }

    pub async fn init_vault_state(&mut self) -> Result<(), JsValue> {
        self.router.vault.init_vault_state().await.map_err(|e| JsValue::from_str(&e.to_string())).expect("init vault state failure");
        Ok(())
    }

    // Note: Need to call init_vault_state first
    pub fn get_vault_state(&mut self) -> JsValue {
        let vault_state = self.router.vault.state;
        to_value(&vault_state).unwrap()
    }

    pub fn get_buy_glp_from_amount(&mut self, to_amount: u64, token_address: &str) -> JsValue {
        let token = WasmRouter::find_token_by_address(self, token_address);
        if !token.is_some() {
            let (amount_out, fee_basis_point) = self.router.vault.state.get_buy_glp_from_amount(U256::from(to_amount), token.unwrap());
            let buy_glp = GetAmountOut {
                amount_out,
                fee_basis_point,
            };
            to_value(&buy_glp).unwrap()
        } else {
            to_value(&{}).unwrap()
        }
    }

    pub fn get_buy_glp_to_amount(&mut self, to_amount: u64, token_address: &str) -> JsValue {
        let token = WasmRouter::find_token_by_address(self, token_address);
        if !token.is_some() {
            let (glp_amount, fee_basis_point) = self.router.vault.state.get_buy_glp_to_amount(&U256::from(to_amount), token.unwrap());
            let buy_glp = GetAmountOut {
                amount_out: glp_amount,
                fee_basis_point,
            };
            to_value(&buy_glp).unwrap()
        } else {
            to_value(&{}).unwrap()
        }
    }

    pub fn get_sell_glp_to_amount(&mut self, to_amount: u64, token_address: &str) -> JsValue {
        let token = WasmRouter::find_token_by_address(self, token_address);
        if !token.is_some() {
            let (amount_out, fee_basis_point) = self.router.vault.state.get_sell_glp_to_amount(U256::from(to_amount), token.unwrap());
            let sell_glp = GetAmountOut {
                amount_out,
                fee_basis_point,
            };
            to_value(&sell_glp).unwrap()
        } else {
            to_value(&{}).unwrap()
        }
    }

    pub fn get_sell_glp_from_amount(&mut self, to_amount: u16, token_address: &str) -> JsValue {
        let token = WasmRouter::find_token_by_address(self, token_address);
        if !token.is_some() {
            let (amount_out, fee_basis_point) = self.router.vault.state.get_sell_glp_from_amount(U256::from(to_amount), token.unwrap());
            let sell_glp = GetAmountOut {
                amount_out,
                fee_basis_point,
            };
            to_value(&sell_glp).unwrap()
        } else {
            to_value(&{}).unwrap()
        }
    }

    pub fn get_fee_basis_points(&mut self,
                                token_weight: u64,
                                token_usdg_amount: u64,
                                usdp_delta: u64,
                                increment: bool, ) -> JsValue {
        let fee_basis_point = self.router.vault.state.get_fee_basis_points(
            token_weight,
            &U256::from(token_usdg_amount),
            &U256::from(usdp_delta),
            increment);
        to_value(&fee_basis_point).unwrap()
    }

    pub fn get_plp_price(&mut self, is_buy: bool) -> JsValue {
        to_value(&self.router.vault.state.get_plp_price(is_buy)).unwrap()
    }

    fn find_token_by_address(&self, token_address: &str) -> Option<&Token> {
        let token: Option<&Token> = None;
        for token_element in self.router.config.tokens.iter() {
            if token_address == token_element.address {
                return Some(&token_element);
            }
        }
        return token;
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetAmountOut {
    pub amount_out: U256,
    pub fee_basis_point: u64,
}

// #[derive(Serialize, Deserialize)]
// pub struct GetAmountOut {
//     pub glp_amount: U256,
//     pub fee_basis_point: u32,
// }

#[wasm_bindgen(start)]
pub fn main() {
    // Set the panic hook to forward messages to console.error
    console_error_panic_hook::set_once();

    // Your other code...
}
