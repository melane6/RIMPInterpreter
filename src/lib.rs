mod utils;
mod lexer;
mod parser;
mod evaluator;
mod debugger;

use wasm_bindgen::prelude::*;
use crate::utils::init_logging;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, testInterpreterWeb!");
}

pub fn initialize_library() {
    init_logging();
}