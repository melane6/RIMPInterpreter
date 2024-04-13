mod debugger;
mod evaluator;
mod lexer;
mod parser;
mod regex;
mod transformations;
mod utils;
mod validator;
mod inverter;
mod error_handler;

use crate::evaluator::Evaluator;
use crate::utils::set_panic_hook;
use std::fmt::Debug;
use wasm_bindgen::prelude::*;

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
pub fn run(program: String) -> String {
    let tokens = lexer::tokenise(program.as_str());
    let result = parser::parse_rimp(tokens);
    match result {
        Ok(program) => {
            let program = program.iter().next().unwrap();
            let mut evaluator = Evaluator::new(program.clone());
            evaluator.interpret_all();
            return format!("{:?}", evaluator);
        }
        Err(e) => {
            return format!("Failed to translate program due to {:?}", e);
        }
    }
}

#[wasm_bindgen]
pub fn initialize_library() {
    set_panic_hook();
}
