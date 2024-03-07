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

use std::fs;
use crate::utils::init_logging;
use wasm_bindgen::prelude::*;
use crate::evaluator::Evaluator;
use crate::interpreter::Interpreter;
use crate::lexer::tokenise;
use crate::parser::parse_rimp;

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
pub fn greet() {
    alert("Hello, RIMPInterpreter!");
}

#[wasm_bindgen]
pub fn tokeniser(input: &str) -> Evaluator {
    let tokens = tokenise(input);
    println!("{:?}", tokens);
    let result = parse_rimp(tokens);
    println!("{:?}", result);
    let mut interpreter = Evaluator::new(vec![]);
    match result {
        Ok(program) => {
            let program = program.iter().next().unwrap();
            interpreter = Evaluator::new(program.clone());
            interpreter.interpret_all();
        }
        Err(e) => {
            println!("Failed to parse program. {:?}", e);
        }
    }
    return interpreter
}

#[wasm_bindgen]
pub fn parse(input: &str) -> String {
    todo!("Implement parser");
}

#[wasm_bindgen]
pub fn evaluate_all(input: &str) -> String {
    todo!("Implement evaluator");
}

#[wasm_bindgen]
pub fn evaluate_step_forwards(input: &str) -> String {
    todo!("Implement evaluator");
}

#[wasm_bindgen]
pub fn evaluate_step_backwards(input: &str) -> String {
    todo!("Implement evaluator");
}



pub fn initialize_library() {
    init_logging();
}
