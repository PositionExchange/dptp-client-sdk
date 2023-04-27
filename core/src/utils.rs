use std::str::FromStr;

use ethers::types::{Address, Bytes};
use tiny_keccak::{Keccak, Hasher};

pub fn _get_function_selector(function_signature: &str) -> [u8; 4] {
    let mut keccak = Keccak::v256();
    let mut output = [0u8; 32];
    let mut selector = [0u8; 4];

    keccak.update(function_signature.as_bytes());
    keccak.finalize(&mut output);

    selector.copy_from_slice(&output[0..4]);
    selector
}

pub fn get_encode_address_and_params(address: &str, function_signature: &str, params: &[ethabi::Token]) -> (Address, Bytes) {
    let address = Address::from_str(address).expect("Failed to parse address");
    let data = encode_selector_and_params(function_signature, params);
    (address, data)
}

pub fn encode_selector_and_params(function_signature: &str, params: &[ethabi::Token]) -> Bytes {
    let selector = _get_function_selector(function_signature);
    let mut encoded = selector.to_vec();
    encoded.extend(ethabi::encode(params));
    Bytes::from(encoded)
}

pub fn get_vault_variable_selector(vault_address: &str, variable_name: &str) -> (Address, Bytes) {
    let address = Address::from_str(vault_address).expect("Failed to parse vault address");
    let fn_selector_raw = _get_function_selector(&format!("{}()", variable_name));
    (address, Bytes::from(fn_selector_raw))
}
