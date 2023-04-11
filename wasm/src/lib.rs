use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::{to_value, from_value};
use core::*;
use std::sync::{Arc, Mutex};

#[wasm_bindgen]
pub struct WasmRouter {
    router: core::Router,//Box<dyn core::Router>,
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

#[wasm_bindgen]
impl WasmRouter {
    #[wasm_bindgen(constructor)]
    pub fn new(chain_id : u64) -> Self {
        let mut router = Router::new();


        router.load_config(chain_id).expect("load_config failed");

        WasmRouter {
            router,
        }
    }

    pub fn load_config(&mut self, chain_id: u64) -> Result<JsValue, JsValue> {
        match self.router.load_config(chain_id) {
            Ok(config) => Ok(to_value(config).unwrap()),
            Err(e) => Err(JsValue::from_str(e)),
        }
    }

    pub fn load_tokens(&self) -> JsValue {
        let tokens = self.router.load_tokens();
        to_value(&tokens).unwrap()
    }

    pub fn set_account(&mut self, account: String) {
        self.router.set_account(account);
    }


    pub async fn fetch_data(&mut self) -> Result<(), JsValue> {
        self.router.fetch_data().await.map_err(|e| JsValue::from_str(&e.to_string())).expect("featch data failure");
        Ok(())
    }
}

