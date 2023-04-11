use super::*;
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_initilize() {
    let mut router = WasmRouter::new(97);
    // expect config to be loaded
    let config = router.load_config(97);
    println!("config {:?}", config);
}

// #[wasm_bindgen_test]
// async fn test_init_vault_state() {
//     let mut router = WasmRouter::new(1);
//     router.init_vault_state().await.expect("init vault state failed");
// }
//
// #[wasm_bindgen_test]
// async fn test_get_vault_state() {
//     let mut router = WasmRouter::new(1);
//     router.init_vault_state().await.expect("init vault state failed");
//     let vault_state = router.get_vault_state();
//     println!("vault_state: {:?}", vault_state);
// }
