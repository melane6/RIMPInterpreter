/// This is the entry point for the wasm module.
/// It contains the functions that are exposed to the Typescript environment.
/// 
/// The functions are:
///    - initialise: This function creates a new Evaluator instance
///    - run: This function takes a string program, tokenizes it, parses it, and interprets it
///    - initialize_library: This function sets up the panic hook for the wasm module
/// 
/// The Structs are:
///   - Evaluator: This struct contains the logic for the RIMPInterpreter
///   - Debugger: This struct contains the logic for the RIMPInterpreter debugger
/// 
/// 

mod lexing;
mod parsing;
mod interpreter;

mod utils;

use crate::interpreter::evaluator::Evaluator;
use crate::utils::set_panic_hook;
use wasm_bindgen::prelude::*;
use crate::lexing::rimp_tokeniser::tokenise;
use crate::parsing::parser::parse_rimp;

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
    let tokens = tokenise(program.as_str());
    let tokens = match tokens {
        Ok(tokens) => tokens,
        Err(e) => {
            return format!("Failed to tokenise program due to {:?}", e);
        }
    };
    let result = parse_rimp(tokens);
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
