use crate::evaluator::RIMPInterpreter;

struct Inverter {
    interpreter: RIMPInterpreter,
    all: bool, // If true, invert til the start of the program else invert a step
}

impl Inverter {
    pub fn new(interpreter: RIMPInterpreter, all: bool) -> Self {
        Inverter {
            interpreter,
            all,
        }
    }

    pub fn invert(&mut self) {
        if self.all {
            while self.interpreter.step() {}
        } else {
            self.interpreter.step();
        }
    }
}