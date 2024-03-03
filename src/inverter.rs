use crate::evaluator::Evaluator;

struct Debugger {
    interpreter: Evaluator,
    all: bool, // If true, invert til the start of the program else invert a step
}

impl Debugger {
    pub fn new(interpreter: Evaluator, all: bool) -> Self {
        Debugger {
            interpreter,
            all,
        }
    }



    pub fn invert(&mut self) {
        // if self.all {
        //     self.interpreter.interpret_all();
        // } else {
        //     self.interpreter.interpret_step();
        // }
    }
}