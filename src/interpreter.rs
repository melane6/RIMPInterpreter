use crate::evaluator::Evaluator;
use crate::parser::Block;

struct Interpreter {
    forward_interpreter: Evaluator,
    temp_interpreter: Evaluator, // Temporary interpreter to debug ?
    debugger_mode: bool,
}

impl Interpreter {
    pub fn new(program: Block) -> Self {
        Interpreter {
            forward_interpreter: Evaluator::new(program.clone()),
            temp_interpreter: Evaluator::new(program.clone()), // maybe not needed
            debugger_mode: false,
        }
    }

    pub fn run(&mut self) { // Run the whole program from start to end
        self.forward_interpreter.interpret_all();
    }

    pub fn set_debugger_mode(&mut self, mode: bool) {
        self.debugger_mode = mode;
    }

    pub fn step_forward(&mut self) { // Run the program one step at a time
        if self.debugger_mode {
            self.forward_interpreter.interpret_step();
        } else {
            panic!("Interpreter is not in debugger mode");
        }
    }

    pub fn step_backward(&mut self) { // Run the program one step at a time in reverse
        if self.debugger_mode {
            self.temp_interpreter = Evaluator::new(self.forward_interpreter.get_reverse_program());
            self.temp_interpreter.interpret_step();
        } else {
            panic!("Interpreter is not in debugger mode");
        }
    }
}