use std::cmp::PartialEq;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::evaluator::Evaluator;
use crate::lexer::tokenise;
use crate::parser::{parse_rimp};

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
    None,           // No debugger mode - run the whole program
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
    pub fn new() -> Self {
        Debugger {
            forward_interpreter: Evaluator::new(vec![]),
            debugger_mode: DebuggerMode::None,
            breakpoints: vec![],
        }
    }

    /// Update the program in the debugger
    /// This will reparse the program and update the interpreter
    pub fn update_program(&mut self, program: String) -> String {
        let tokens = tokenise(program.clone().as_str());
        let result = parse_rimp(tokens);
        match result {
            Ok(program) => {
                let program = program.iter().next().unwrap();
                self.forward_interpreter = Evaluator::new(program.clone());
                return format!("{:?}", program);
            }
            Err(e) => {
                return format!("Failed to translate program due to {:?}", e);
            }
        }
    }

    pub fn set_breakpoints(&mut self, breakpoints: Vec<usize>) { // Set the breakpoints for the debugger
        self.breakpoints = breakpoints;
    }

    pub fn get_breakpoints(&self) -> Vec<usize> { // Get the breakpoints for the debugger
        self.breakpoints.clone()
    }

    pub fn set_debugger_mode(&mut self, mode: DebuggerMode) { // Set the debugger mode
        self.debugger_mode = mode;
    }

    pub fn get_debugger_mode(&self) -> DebuggerMode { // Get the debugger mode
        self.debugger_mode.clone()
    }

    pub fn run(&mut self) { // Run the whole program from start to end
        if self.debugger_mode == DebuggerMode::None {
            self.forward_interpreter.interpret_all();
        }
    }

    pub fn step_forward(&mut self) { // Run the program one step at a time
        if self.debugger_mode == DebuggerMode::StepForward {
            self.forward_interpreter.interpret_step();
            println!("{:?}", self.forward_interpreter.get_program());
        } else {
            panic!("Interpreter is not in debugger mode");
        }
    }

    pub fn step_backward(&mut self) { // Run the program one step at a time in reverse
        if self.debugger_mode == DebuggerMode::StepBackward {
            panic!("Step backward is not implemented yet");
        } else {
            panic!("Interpreter is not in debugger mode");
        }
    }
}