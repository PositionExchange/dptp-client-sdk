extern crate console_error_panic_hook;
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::{to_value, from_value};
use core::*;
use std::sync::{Arc, Mutex};
use std::panic;
use wasm_bindgen_test::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        let mut router = Router::new();


        router.load_config(chain_id).expect("load_config failed");

        // panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_error_panic_hook::set_once();
        // set_panic_hook();
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

//
// pub fn set_panic_hook() {
//     // When the `console_error_panic_hook` feature is enabled, we can call the
//     // `set_panic_hook` function at least once during initialization, and then
//     // we will get better error messages if our code ever panics.
//     //
//     // For more details see
//     // https://github.com/rustwasm/console_error_panic_hook#readme
//     #[cfg(feature = "console_error_panic_hook")]
//     console_error_panic_hook::set_once();
// }
//

#[wasm_bindgen_test]
fn fetch_data(){
    let mut router = WasmRouter::new(97);
    router.fetch_data();
}


wasm_bindgen_test_configure!(run_in_browser);
// #[wasm_bindgen_test]
// mod test {
//
//     #[wasm_bindgen_test]
//     fn fetch_data(){
//         let mut router = WasmRouter::new(97);
//         router.fetch_data();
//     }
// }

