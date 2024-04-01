mod evaluator;
mod lexer;
mod parser;
mod regex;
mod transformations;
mod utils;
mod validator;
mod debugger;

use std::fmt::Debug;
use crate::utils::set_panic_hook;
use wasm_bindgen::prelude::*;
use crate::evaluator::Evaluator;


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    fn console_log(s: &str);
}

#[wasm_bindgen]
pub fn initialise() -> Evaluator {
    Evaluator::new(vec![])
}

#[wasm_bindgen]
pub fn initialize_library() {
    set_panic_hook();
}
