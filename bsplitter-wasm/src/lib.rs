mod utils;

use wasm_bindgen::prelude::*;
extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! console_log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
extern "C" {
}

#[wasm_bindgen]
pub fn greet() {
    console_log!("Hello, bsplitter-wasm! From a worker");
}
