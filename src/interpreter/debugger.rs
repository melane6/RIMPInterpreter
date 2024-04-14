use crate::interpreter::evaluator::Evaluator;
use crate::lexing::rimp_tokeniser::tokenise;
use crate::parsing::parser::parse_rimp;
use std::cmp::PartialEq;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::parsing::transformations::translate_program;

#[wasm_bindgen]
#[derive(PartialEq, Debug, Clone)]
pub enum DebuggerMode {
    StepForward,
    // Evaluate one step forward
    StepBackward,
    // Evaluate one step backward
    StepInto,
    // Step into a function call
    StepOut,
    // Step out of a function call
    StepOver,
    // Step over a function call
    None, // No debugger mode - run the whole program
}

#[wasm_bindgen]
pub(crate) struct Debugger {
    forward_interpreter: Evaluator,
    debugger_mode: DebuggerMode,
    breakpoints: Vec<usize>,
}

#[wasm_bindgen]
impl Debugger {
    /// Create a new debugger instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Debugger {
            forward_interpreter: Evaluator::new(vec![]),
            debugger_mode: DebuggerMode::None,
            breakpoints: vec![0],
        }
    }

    /// Update the program in the debugger
    /// This will reparse the program and update the interpreter
    ///
    #[wasm_bindgen]
    pub fn update_program(&mut self, program: String) -> String {
        let tokens = tokenise(program.clone().as_str());
        let tokens = match tokens {
            Ok(tokens) => tokens,
            Err(e) => {
                return format!("Failed to tokenise program due to {:?}", e);
            }
        };
        let result = parse_rimp(tokens);
        match result {
            Ok(program) => {
                let program = translate_program(program.iter().next().unwrap().clone());
                self.forward_interpreter = Evaluator::new(program.clone());
                self.forward_interpreter.set_original_program();
                return self.forward_interpreter.get_program();
            }
            Err(e) => {
                return format!("Failed to parse/lex program due to {:?}", e);
            }
        }
    }

    #[wasm_bindgen]
    pub fn reset(&mut self, program: String) {
        // Reset the debugger
        self.forward_interpreter.reset();
        self.update_program(program);
        self.debugger_mode = DebuggerMode::None;
        self.breakpoints = vec![0];
    }

    pub fn get_environment(&self) -> String {
        format!("{}", self.forward_interpreter.get_environment())
    }

    pub fn get_program(&self) -> String {
        format!("{}", self.forward_interpreter.get_program())
    }

    pub fn get_back_program(&self) -> String {
        format!("{}", self.forward_interpreter.get_back_program())
    }

    pub fn set_breakpoints(&mut self, breakpoints: Vec<usize>) {
        // Set the breakpoints for the debugger
        self.breakpoints = breakpoints;
    }

    pub fn get_breakpoints(&self) -> Vec<usize> {
        // Get the breakpoints for the debugger
        self.breakpoints.clone()
    }
    
    pub fn swap(&mut self) {
        self.forward_interpreter.swap();
    }

    pub fn set_debugger_mode(&mut self, mode: DebuggerMode) {
        // Set the debugger mode
        if mode == DebuggerMode::None {
            self.forward_interpreter.reset();
        }
        self.debugger_mode = mode;
    }

    pub fn get_debugger_mode(&self) -> DebuggerMode {
        // Get the debugger mode
        self.debugger_mode.clone()
    }

    pub fn run(&mut self) {
        // Run the whole program from start to end
        if self.debugger_mode == DebuggerMode::None {
            self.forward_interpreter.interpret_all();
        }
    }

    pub fn step_forward(&mut self) -> String {
        // Run the program one step at a time
        match self.debugger_mode {
            DebuggerMode::StepForward => {
                if self.forward_interpreter.get_program().is_empty() {
                    return "Program is empty".to_string();
                }
                self.forward_interpreter.interpret_step();
                return self.forward_interpreter.get_environment();
            }
            DebuggerMode::StepBackward => {
                self.debugger_mode = DebuggerMode::StepForward;
                self.forward_interpreter.swap();
                return self.forward_interpreter.get_environment();
            }
            DebuggerMode::StepInto => {
                return "Need to implement".to_string();
            }
            DebuggerMode::StepOut => {
                return "Need to implement".to_string();
            }
            DebuggerMode::StepOver => {
                return "Need to implement".to_string();
            }
            DebuggerMode::None => {
                self.forward_interpreter.reset();
                self.debugger_mode = DebuggerMode::StepForward;
                self.breakpoints = vec![0];
                return self.forward_interpreter.get_environment();
            }
        }
    }

    pub fn step_backward(&mut self) -> String {
        // Run the program one step at a time in reverse
        match self.debugger_mode {
            DebuggerMode::StepForward => {
                self.debugger_mode = DebuggerMode::StepBackward;
                self.forward_interpreter.swap();
                println!("Swapped");
                println!("{:?}", self.forward_interpreter.get_program());
                return self.forward_interpreter.get_environment();
            }
            DebuggerMode::StepBackward => {
                if self.forward_interpreter.get_back_program().is_empty() {
                    return "You are at the start of the program".to_string();
                }
                self.forward_interpreter.interpret_step_back();
                return self.forward_interpreter.get_environment();
            }
            DebuggerMode::StepInto => {
                return "Need to implement".to_string();
            }
            DebuggerMode::StepOut => {
                return "Need to implement".to_string();
            }
            DebuggerMode::StepOver => {
                return "Need to implement".to_string();
            }
            DebuggerMode::None => {
                return "Can't step back yet. Please step forward first".to_string();
            }
        }
    }
}
