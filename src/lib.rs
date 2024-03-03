mod interpreter;
mod error_handler;
mod evaluator;
mod inverter;
mod lexer;
mod parser;
mod regex;
mod transformations;
mod utils;
mod validator;

use crate::utils::init_logging;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, RIMPInterpreter!");
}

pub fn initialize_library() {
    init_logging();
}
