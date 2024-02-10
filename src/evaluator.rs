use std::collections::HashMap;


struct Counter {
    value: i32,
}

impl Counter {
    fn new() -> Counter {
        Counter { value: 0 }
    }

    fn increment(&mut self) {
        self.value += 1;
    }
}

type Environment = HashMap<String, i64>;
struct RIMPInterpreter {
    counter: Counter,
    environment: Environment,
}

impl RIMPInterpreter {
    fn new() -> RIMPInterpreter {
        RIMPInterpreter {
            counter: Counter::new(),
            environment: Environment::new(),
        }
    }

    fn interpret_all(&mut self, program: &str) -> i64 {
        todo!("Implement the interpret method")
    }

    fn interpret_step(&mut self, program: &str) -> i64 {
        todo!("Implement the interpret step method")
    }
}