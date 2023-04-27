#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(not(target_arch = "wasm32"))]
fn log(s: &str) {
    println!("{}", s);
}

pub fn print(s: &str) {
    log(format!("RUST:: {}", s).as_str());
}

#[macro_export]
macro_rules! p {
    ($($t:tt)*) => (crate::print(&format_args!($($t)*).to_string()));
}
