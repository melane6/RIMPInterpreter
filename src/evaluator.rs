use crate::lexer::tokenise;
use crate::parser::{parse_rimp, ArithmeticExpr, Block, BooleanExpr, Stmt};
use crate::validator::TypeChecker;
use std::collections::HashMap;
use std::fs;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug)]
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

type Environment = HashMap<String, Vec<i64>>;

#[wasm_bindgen]
pub(crate) struct Evaluator {
    counter: Counter,
    environment: Environment,
    functions: HashMap<String, (Environment, Block)>,
    function_name: Vec<String>,
    back_trace: Block, // Reverse the steps to get the start of the program
    program: Block,
    original_stmt: bool
}

// For pretty printing and debugging or displaying the state of the interpreter
impl std::fmt::Debug for Evaluator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "environment: \n{}\n back_trace: \n{} ",
            self.get_environment(),
            self.get_back_program().to_string()
        )
    }
}

#[wasm_bindgen]
impl Evaluator {
    pub(crate) fn new(program: Block) -> Evaluator {
        // Create RIMPInterpreter with necessary environment and functions for the program
        Evaluator {
            counter: Counter::new(),
            environment: Environment::new(),
            functions: Default::default(),
            function_name: vec!["main".to_string()],
            back_trace: vec![],
            program,
            original_stmt: true
        }
    }

    pub(crate) fn reset(&mut self) {
        // Reset the interpreter to the initial state
        self.counter = Counter::new();
        self.environment = Environment::new();
        self.functions = Default::default();
        self.function_name = vec!["main".to_string()];
        self.back_trace = vec![];
    }

    pub(crate) fn get_environment(&self) -> String {
        let mut program = String::new();
        for (key, values) in self.environment.clone() {
            let mut value = String::new();
            for i in 0..values.len() {
                if (i == values.len() - 1 || values.len() == 1) {
                    value.push_str(format!("{}", values[i]).as_str());
                } else {
                    value.push_str(format!("{}, ", values[i]).as_str());
                }
            }
            program.push_str(format!("\t{}: {}\n", key, value).as_str());
        }
        program
    }

    pub(crate) fn get_program(&self) -> String {
        let mut program = String::new();
        // check if it is the last statement to not inclued ;
        for i in 0..self.program.len() {
            if i == self.program.len() - 1 {
                program.push_str(format!("{}", self.program[i].to_string()).as_str());
            } else {
                program.push_str(format!("{};\n", self.program[i].to_string()).as_str());
            }
        }
        program
    }

    pub(crate) fn get_back_program(&self) -> String {
        let mut program = String::new();
        for stmt in self.back_trace.clone() {
            program.push_str(format!("{};\n", stmt.to_string()).as_str());
        }
        program
    }
    pub(crate) fn write(&mut self) {
        self.program = self.eval_small_step_block(self.program.clone());
        log::debug!("{:?}", self) // Print the state of the interpreter
    }

    pub(crate) fn empty_program(&mut self) -> bool {
        self.program.is_empty()
    }

    pub(crate) fn interpret_all(&mut self) {
        self.eval_block(self.program.clone());
        println!("{:?}", self.environment);
        println!("{:?}", self.functions);
    }

    pub(crate) fn interpret_step(&mut self) {
        self.program = self.eval_small_step_block(self.program.clone());
        log::debug!("{:?}", self) // Print the state of the interpreter
    }

    pub(crate) fn step_back(&mut self) {
        self.back_trace = self.eval_back_step_block();
        log::debug!("{:?}", self) // Print the state of the interpreter
    }

    pub(crate) fn get_reverse_program(&mut self) -> Block {
        self.back_trace.clone()
    }

    fn get_function_env(&mut self, function_name: String, var: String) -> i64 {
        if function_name == "main" {
            self.environment
                .get(&var)
                .unwrap()
                .clone()
                .first()
                .unwrap()
                .clone()
        } else {
            self.functions
                .get(self.function_name.last().unwrap())
                .unwrap()
                .0
                .get(&var)
                .unwrap()
                .clone().first().unwrap().clone()
        }
    }
    
    fn set_variable_value(&mut self, function_name: String, var: String, value: i64) {
        if function_name == "main" {
            let mut values = self.environment.get(&var).unwrap().clone();
            values.insert(0, value);
            self.environment.insert(var.clone(), values);
        } else {
            let mut values = self
                .functions
                .get(self.function_name.last().unwrap())
                .unwrap()
                .0
                .get(&var)
                .unwrap()
                .clone();
            values.insert(0, value);
            self.functions
                .get_mut(self.function_name.last().unwrap())
                .unwrap()
                .0
                .insert(var.clone(), values);
        }
    }
    
    fn check_overlapping(&mut self, boolean_expr : BooleanExpr ,then_block: Block, else_block: Block) -> Vec<String> {
        let mut validator = TypeChecker::new();
        // get environment name
        let env_name = self.function_name.last().unwrap();
        let condition_vars =
            validator.check_boolean_expr(boolean_expr.clone(), env_name.clone());
        // check if the condition variables are in the environment
        let then_vars = validator.check(then_block.clone(), env_name.clone());
        let else_vars = validator.check(else_block.clone(), env_name.clone());
        let overlapping_vars = validator.check_overlapping_vars(condition_vars, then_vars, else_vars);
        overlapping_vars
    }

    fn modify_blocks(&mut self, overlapping_vars: Vec<String>, then_block: Block, else_block: Block) -> (Block, Block) {
        if !overlapping_vars.is_empty() {
            let mut mapping = HashMap::new();
            for var in overlapping_vars {
                // copy the value of the variable to the environment
                let function_name = self.function_name.last().unwrap().clone();
                let value = self.get_function_env(function_name.clone(), var.clone());
                if self.function_name.last().unwrap() == "main" {
                    let new_var = format!("{}'", var);
                    mapping.insert(var, new_var.clone());
                    self.set_variable_value(function_name.clone(), new_var.clone(), value);
                    self.back_trace
                        .push(Stmt::Assign(new_var, ArithmeticExpr::Num(value)));
                } else {
                    let new_var = format!("{}'", var);
                    mapping.insert(var, new_var.clone());
                    self.set_variable_value(function_name.clone(), new_var.clone(), value);
                    self.back_trace
                        .push(Stmt::Assign(new_var, ArithmeticExpr::Num(value)));
                }
            }
            // remap all references to the variable in the then and else block
            return (self.remap_vars(then_block, mapping.clone()),self.remap_vars(else_block, mapping.clone()))
        }
        return (then_block, else_block)
    }
    fn eval_small_step_arithmetic(&mut self, expr: ArithmeticExpr) -> ArithmeticExpr {
        match expr {
            ArithmeticExpr::Num(n) => ArithmeticExpr::Num(n),
            ArithmeticExpr::LocalID(id) => {
                let function_name = self.function_name.last().unwrap().clone();
                if self.environment.contains_key(&id) {
                    ArithmeticExpr::Num(
                        self.get_function_env(function_name.clone(), id.clone())
                    )
                } else {
                    ArithmeticExpr::Num(0) // default value
                }
            }
            ArithmeticExpr::GlobalID(id) => ArithmeticExpr::Num(
                self.get_function_env("main".to_string(), id.clone())
            ),
            ArithmeticExpr::Aop(op, a1, a2) => match *a1 {
                ArithmeticExpr::Num(num) => {
                    let new_a1 = self.eval_small_step_arithmetic(*a2);
                    ArithmeticExpr::Aop(op, Box::new(ArithmeticExpr::Num(num)), Box::new(new_a1))
                }
                _ => match *a2 {
                    ArithmeticExpr::Num(num) => {
                        let new_a2 = self.eval_small_step_arithmetic(*a1);
                        ArithmeticExpr::Aop(
                            op,
                            Box::new(new_a2),
                            Box::new(ArithmeticExpr::Num(num)),
                        )
                    }
                    _ => {
                        let n1 = self.eval_arithmetic_expr(*a1);
                        let n2 = self.eval_arithmetic_expr(*a2);
                        match op.as_str() {
                            "+" => ArithmeticExpr::Num(n1 + n2),
                            "-" => ArithmeticExpr::Num(n1 - n2),
                            "*" => ArithmeticExpr::Num(n1 * n2),
                            "/" => ArithmeticExpr::Num(n1 / n2),
                            "%" => ArithmeticExpr::Num(n1 % n2),
                            _ => panic!("unknown op"),
                        }
                    }
                },
            },
            ArithmeticExpr::Call(fn_id, mut args) => {
                if self.function_name.contains(&fn_id) {
                    panic!("Recursion is not allowed unless tail-recursive");
                }
                if !args.is_empty() {
                    for arg in self.functions.get(&fn_id).unwrap().clone().0 {
                        let value = self.eval_arithmetic_expr(args[0].clone());
                        println!("fn {:?}", self.function_name);
                        self.functions
                            .get_mut(&fn_id)
                            .unwrap()
                            .0
                            .insert(arg.0, vec![value]); // update the environment with evaluated argument
                        args.remove(0);
                        break; // Evaluates only one argument at a time
                    }
                    ArithmeticExpr::Call(fn_id, args)
                } else {
                    // Evaluate the function block and return the value here
                    // TODO: Step through the function block if step into is enabled?
                    ArithmeticExpr::Num(
                        self.eval_arithmetic_expr(ArithmeticExpr::Call(fn_id, args)),
                    )
                }
            }
        }
    }

    fn eval_arithmetic_expr(&mut self, expr: ArithmeticExpr) -> i64 {
        match expr {
            ArithmeticExpr::Num(n) => n,
            ArithmeticExpr::LocalID(id) => {
                if self.function_name.last().unwrap() == "main" {
                    self.environment
                        .get(&id)
                        .unwrap()
                        .clone()
                        .first()
                        .unwrap()
                        .clone()
                } else {
                    let function_env = self
                        .functions
                        .get(self.function_name.last().unwrap())
                        .unwrap()
                        .0
                        .clone();
                    function_env
                        .get(&id)
                        .unwrap()
                        .clone()
                        .first()
                        .unwrap()
                        .clone()
                }
            }
            ArithmeticExpr::GlobalID(id) => self
                .environment
                .get(&id)
                .unwrap()
                .clone()
                .first()
                .unwrap()
                .clone(),
            ArithmeticExpr::Aop(op, a1, a2) => {
                let n1 = self.eval_arithmetic_expr(*a1);
                let n2 = self.eval_arithmetic_expr(*a2);
                match op.as_str() {
                    "+" => n1 + n2,
                    "-" => n1 - n2,
                    "*" => n1 * n2,
                    "/" => n1 / n2,
                    "%" => n1 % n2,
                    _ => panic!("unknown op"),
                }
            }
            ArithmeticExpr::Call(fn_id, mut args) => {
                if self.function_name.contains(&fn_id) {
                    panic!("Recursion is not allowed");
                }
                for arg in self.functions.get(&fn_id).unwrap().clone().0 {
                    if args.is_empty() {
                        break;
                    }
                    let value = self.eval_arithmetic_expr(args[0].clone());
                    println!("fn {:?}", self.function_name);
                    self.functions
                        .get_mut(&fn_id)
                        .unwrap()
                        .0
                        .insert(arg.0, vec![value]);
                    args.remove(0);
                }
                self.function_name.push(fn_id.clone());
                for stmt in self.functions.get(&fn_id).unwrap().1.clone() {
                    self.eval_stmt(stmt.clone());
                }
                self.function_name.remove(0);
                self.get_return_value(fn_id)
            }
        }
    }

    fn get_return_value(&mut self, function_name: String) -> i64 {
        if function_name == "main" {
            self.environment
                .get("return")
                .unwrap()
                .clone()
                .first()
                .unwrap()
                .clone()
        } else {
            self.functions
                .get(&function_name)
                .unwrap()
                .0
                .get("return")
                .unwrap()
                .clone()
                .first()
                .unwrap()
                .clone()
        }
    }

    fn eval_small_step_boolean_expr(&mut self, expr: BooleanExpr) -> BooleanExpr {
        match expr {
            BooleanExpr::Bop(op, a1, a2) => match a1 {
                ArithmeticExpr::Num(n1) => match a2 {
                    ArithmeticExpr::Num(n2) => match op.as_str() {
                        "==" => {
                            if n1 == n2 {
                                BooleanExpr::True
                            } else {
                                BooleanExpr::False
                            }
                        }
                        "!=" => {
                            if n1 != n2 {
                                BooleanExpr::True
                            } else {
                                BooleanExpr::False
                            }
                        }
                        "<" => {
                            if n1 < n2 {
                                BooleanExpr::True
                            } else {
                                BooleanExpr::False
                            }
                        }
                        "<=" => {
                            if n1 <= n2 {
                                BooleanExpr::True
                            } else {
                                BooleanExpr::False
                            }
                        }
                        ">" => {
                            if n1 > n2 {
                                BooleanExpr::True
                            } else {
                                BooleanExpr::False
                            }
                        }
                        ">=" => {
                            if n1 >= n2 {
                                BooleanExpr::True
                            } else {
                                BooleanExpr::False
                            }
                        }
                        _ => panic!("unknown op"),
                    },
                    _ => {
                        let new_a2 = self.eval_small_step_arithmetic(a2);
                        BooleanExpr::Bop(op, ArithmeticExpr::Num(n1), new_a2)
                    }
                },
                _ => {
                    let new_a1 = self.eval_small_step_arithmetic(a1);
                    BooleanExpr::Bop(op, new_a1, a2.clone())
                }
            },
            BooleanExpr::Lop(op, b1, b2) => match *b1 {
                BooleanExpr::True | BooleanExpr::False => match *b2 {
                    BooleanExpr::True | BooleanExpr::False => match op.as_str() {
                        "&&" => {
                            if self.eval_boolean_expr(*b1.clone())
                                && self.eval_boolean_expr(*b2.clone())
                            {
                                BooleanExpr::True
                            } else {
                                BooleanExpr::False
                            }
                        }
                        "||" => {
                            if self.eval_boolean_expr(*b1.clone())
                                || self.eval_boolean_expr(*b2.clone())
                            {
                                BooleanExpr::True
                            } else {
                                BooleanExpr::False
                            }
                        }
                        _ => BooleanExpr::False,
                    },
                    _ => {
                        let new_b2 = self.eval_small_step_boolean_expr(*b2.clone());
                        BooleanExpr::Lop(op, b1, Box::new(new_b2))
                    }
                },
                _ => {
                    let new_b1 = self.eval_small_step_boolean_expr(*b1.clone());
                    BooleanExpr::Lop(op, Box::new(new_b1), b2)
                }
            },
            BooleanExpr::True => BooleanExpr::True,
            BooleanExpr::False => BooleanExpr::False,
        }
    }
    fn eval_boolean_expr(&mut self, expr: BooleanExpr) -> bool {
        match expr {
            BooleanExpr::Bop(op, a1, a2) => {
                let n1 = self.eval_arithmetic_expr(a1);
                let n2 = self.eval_arithmetic_expr(a2);
                match op.as_str() {
                    "==" => n1 == n2,
                    "!=" => n1 != n2,
                    "<" => n1 < n2,
                    "<=" => n1 <= n2,
                    ">" => n1 > n2,
                    ">=" => n1 >= n2,
                    _ => panic!("unknown op"),
                }
            }
            BooleanExpr::Lop(op, b1, b2) => {
                let n1 = self.eval_boolean_expr(*b1);
                let n2 = self.eval_boolean_expr(*b2);
                match op.as_str() {
                    "&&" => n1 && n2,
                    "||" => n1 || n2,
                    _ => panic!("unknown op"),
                }
            }
            BooleanExpr::True => true,
            BooleanExpr::False => false,
        }
    }

    fn eval_small_step_stmt(&mut self, stmt: Stmt) -> Stmt {
        match stmt {
            Stmt::Skip => {
                self.back_trace.push(Stmt::Skip);
                self.original_stmt = true;
                Stmt::Skip
            }
            Stmt::Assign(id, a1) => {
                let new_a1 = self.eval_small_step_arithmetic(a1.clone());
                self.original_stmt = false;
                if new_a1 == a1 {
                    let original_value =
                        self.eval_small_step_arithmetic(ArithmeticExpr::LocalID(id.clone()));
                    println!("original value {:?}", original_value);
                    self.back_trace
                        .push(Stmt::Unassign(id.clone(), original_value)); // Add the original statement to the back trace
                    self.eval_stmt(Stmt::Assign(id.clone(), new_a1.clone()));
                    Stmt::Skip
                } else {
                    Stmt::Assign(id, new_a1)
                }
            }
            Stmt::Unassign(id, value) => {
                // Remove the top value of the id
                let new_a1 = self.eval_small_step_arithmetic(value.clone());
                self.original_stmt = false;
                if new_a1 == value {
                    self.back_trace
                        .push(Stmt::Assign(id.clone(), value)); // Add the original statement to the back trace
                    self.eval_stmt(Stmt::Unassign(id.clone(), new_a1.clone()));
                    Stmt::Skip
                } else {
                    Stmt::Assign(id, new_a1)
                }
            }
            Stmt::If(boolean, mut then_block, mut else_block) => {
                if self.original_stmt {
                    let overlapping_vars = self.check_overlapping(boolean.clone(), then_block.clone(), else_block.clone());
                    (then_block, else_block) = self.modify_blocks(overlapping_vars, then_block.clone(), else_block.clone());
                    self.back_trace.push(Stmt::If(
                        boolean.clone(),
                        then_block.clone(),
                        else_block.clone(),
                    ));
                    Stmt::If(boolean.clone(), then_block.clone(), else_block.clone())
                } else {
                    let new_boolean = self.eval_small_step_boolean_expr(boolean.clone());
                    self.original_stmt = false;
                    if new_boolean == boolean {
                        Stmt::If(boolean, then_block, else_block)
                    } else {
                        self.back_trace.push(Stmt::If(
                            boolean.clone(),
                            then_block.clone(),
                            else_block.clone(),
                        )); // Add the original statement to the back trace
                        Stmt::If(new_boolean, then_block, else_block)
                    }
                }
            }
            Stmt::While(boolean, mut block) => {
                if self.original_stmt {
                    self.back_trace.push(Stmt::While(boolean.clone(), block.clone()));
                }
                block.push(Stmt::While(boolean.clone(), block.clone()));
                self.original_stmt = false;
                self.back_trace
                    .push(Stmt::While(boolean.clone(), block.clone()));
                Stmt::If(boolean, block, vec![Stmt::Skip])
            }
            Stmt::Function(function_name, args, block) => {
                self.back_trace.push(Stmt::Function(
                    function_name.clone(),
                    args.clone(),
                    block.clone(),
                ));
                self.eval_stmt(Stmt::Function(
                    function_name.clone(),
                    args.clone(),
                    block.clone(),
                ));
                Stmt::Skip
            }
            Stmt::Return(expr) => {
                if self.original_stmt {
                    self.back_trace.push(Stmt::Return(expr.clone()));
                }
                let value = self.eval_small_step_arithmetic(expr.clone());
                if value == expr.clone() {
                    self.eval_stmt(Stmt::Return(expr.clone()));
                    Stmt::Skip
                } else {
                    self.back_trace.push(Stmt::Return(expr.clone()));
                    Stmt::Return(value)
                }
            }
            _ => Stmt::Skip,
        }
    }

    fn eval_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Assign(id, expr) => {
                if self.function_name.last().unwrap() == "main" {
                    let value = self.eval_arithmetic_expr(expr.clone());
                    if self.environment.contains_key(&id) {
                        let mut values = self.environment.get(&id).unwrap().clone();
                        values.insert(0, value);
                        self.environment.insert(id.clone(), values);
                    } else {
                        let mut values = vec![value];
                        self.environment.insert(id.clone(), values);
                    }
                } else {
                    let value = self.eval_arithmetic_expr(expr.clone());
                    // check if the variable is in the function environment
                    if self
                        .functions
                        .get(self.function_name.last().unwrap())
                        .unwrap()
                        .0
                        .contains_key(&id)
                    {
                        let mut values = self
                            .functions
                            .get(self.function_name.last().unwrap())
                            .unwrap()
                            .0
                            .get(&id)
                            .unwrap()
                            .clone();
                        values.insert(0, value);
                        self.functions
                            .get_mut(self.function_name.last().unwrap())
                            .unwrap()
                            .0
                            .insert(id.clone(), values);
                    } else {
                        let mut values = vec![value];
                        self.functions
                            .get_mut(self.function_name.last().unwrap())
                            .unwrap()
                            .0
                            .insert(id.clone(), values);
                    }
                }
                // todo!("Implement the back trace for assign statement")
            }
            Stmt::Skip => {}
            Stmt::Unassign(id, value) => {
                if self.function_name.last().unwrap() == "main" {
                    let value = self.eval_arithmetic_expr(value.clone());
                    let mut values = self.environment.get(&id).unwrap().clone();
                    values.remove(0);
                    self.environment.insert(id.clone(), values);
                } else {
                    let value = self.eval_arithmetic_expr(value.clone());
                    let mut values = self
                        .functions
                        .get(self.function_name.last().unwrap())
                        .unwrap()
                        .0
                        .get(&id)
                        .unwrap()
                        .clone();
                    values.remove(0);
                    self.functions
                        .get_mut(self.function_name.last().unwrap())
                        .unwrap()
                        .0
                        .insert(id.clone(), values);
                }
            }
            Stmt::If(boolean_expr, mut then_block, mut else_block) => {
                let overlapping_vars = self.check_overlapping(boolean_expr.clone(), then_block.clone(), else_block.clone());
                (then_block, else_block) = self.modify_blocks(overlapping_vars, then_block.clone(), else_block.clone());
                if self.eval_boolean_expr(boolean_expr.clone()) {
                    self.eval_block(then_block.clone());
                } else {
                    self.eval_block(else_block.clone());
                }
                self.back_trace
                    .push(Stmt::If(boolean_expr, then_block, else_block));
            }
            Stmt::While(boolean, block) => {
                while self.eval_boolean_expr(boolean.clone()) {
                    self.eval_block(block.clone());
                }
                self.back_trace
                    .push(Stmt::While(boolean.clone(), block.clone()));
            }
            Stmt::Function(function_name, args, block) => {
                let mut function_env = Environment::new();
                for arg in args {
                    function_env.insert(arg, vec![0]); // default value
                }
                function_env.insert("return".to_string(), vec![0]); // default value for return value
                self.functions.insert(function_name, (function_env, block));
            }
            Stmt::Return(expr) => {
                if self.function_name.last().unwrap() == "main" {
                    let value = self.eval_arithmetic_expr(expr);
                    let mut values = self.environment.get("return").unwrap().clone();
                    values.insert(0, value);
                    self.environment.insert("return".parse().unwrap(), values);
                } else {
                    let value = self.eval_arithmetic_expr(expr);
                    let mut values = self
                        .functions
                        .get(self.function_name.last().unwrap())
                        .unwrap()
                        .0
                        .get("return")
                        .unwrap()
                        .clone();
                    values.insert(0, value);
                    self.functions
                        .get_mut(self.function_name.last().unwrap())
                        .unwrap()
                        .0
                        .insert("return".parse().unwrap(), values);
                }
            }
            _ => {}
        }
    }

    fn eval_back_step_block(&mut self) -> Block {
        // Evaluate one step backward in the pass in block
        let mut new_block = self.back_trace.clone();
        // Remove the last statement from the back trace
        let remove_statement = self.back_trace.remove(new_block.len() - 1);
        // Insert the removed statement back to the program
        match remove_statement {
            Stmt::Skip
            | Stmt::Assign(_, _)
            | Stmt::If(_, _, _)
            | Stmt::While(_, _)
            | Stmt::Function(_, _, _)
            | Stmt::Return(_) => {
                self.program.insert(0, remove_statement.clone()); // Don't insert Unassign
            }
            _ => {}
        }
        match remove_statement {
            Stmt::Unassign(id, value) => {
                let value = self.eval_arithmetic_expr(value); // Value should be Num
                let mut values = self.environment.get(&id).unwrap().clone();
                values.insert(0, value);
                if self.function_name.last().unwrap() == "main" {
                    self.environment.insert(id.clone(), values);
                } else {
                    self.functions
                        .get_mut(self.function_name.last().unwrap())
                        .unwrap()
                        .0
                        .insert(id.clone(), values);
                }
            }
            _ => {}
        }
        new_block
    }

    fn eval_small_step_block(&mut self, block: Block) -> Block {
        let mut new_block = block.clone();
        let remove_statement = new_block.remove(0);
        let new_stmt = self.eval_small_step_stmt(block[0].clone());
        if new_stmt == Stmt::Skip {
            new_block
        } else if remove_statement == new_stmt {
            // No change
            match new_stmt {
                Stmt::If(exp, then_block, else_block) => {
                    let mut if_stat_block;
                    if self.eval_boolean_expr(exp) {
                        if_stat_block = then_block.clone();
                        if_stat_block.extend(new_block);
                    } else {
                        if_stat_block = else_block.clone();
                        if_stat_block.extend(new_block);
                    }
                    if_stat_block
                }
                _ => new_block,
            }
        } else {
            new_block.insert(0, new_stmt);
            new_block
        }
    }
    fn eval_block(&mut self, block: Block) {
        for stmt in block {
            self.eval_stmt(stmt);
        }
    }

    fn remap_vars(&mut self, block: Vec<Stmt>, mapping: HashMap<String, String>) -> Block {
        let mut new_block = vec![];
        for stmt in block {
            match stmt {
                Stmt::Assign(id, value) => {
                    let arith_expr = self.remap_vars_arithmetic(value, mapping.clone());
                    for (old, new) in mapping.iter() {
                        if id == *old {
                            new_block.push(Stmt::Assign(new.clone(), arith_expr.clone()));
                        }
                    }
                }
                Stmt::If(boolean, then_block, else_block) => {
                    let new_boolean = self.remap_vars_bool(boolean, mapping.clone());
                    let new_then_block = self.remap_vars(then_block, mapping.clone());
                    let new_else_block = self.remap_vars(else_block, mapping.clone());
                    new_block.push(Stmt::If(new_boolean, new_then_block, new_else_block));
                }
                Stmt::While(boolean_expr, block_while) => {
                    let new_boolean = self.remap_vars_bool(boolean_expr, mapping.clone());
                    let block = self.remap_vars(block_while, mapping.clone());
                    new_block.push(Stmt::While(new_boolean, block));
                }
                Stmt::Function(function_name, args, fn_block) => {
                    let block = self.remap_vars(fn_block, mapping.clone());
                    new_block.push(Stmt::Function(function_name, args, block));
                }
                _ => {}
            }
        }
        new_block
    }

    fn remap_vars_arithmetic(
        &mut self,
        expr: ArithmeticExpr,
        mapping: HashMap<String, String>,
    ) -> ArithmeticExpr {
        match expr {
            ArithmeticExpr::Num(n) => ArithmeticExpr::Num(n),
            ArithmeticExpr::LocalID(id) => {
                for (old, new) in mapping.iter() {
                    if id == *old {
                        return ArithmeticExpr::LocalID(new.clone());
                    }
                }
                ArithmeticExpr::LocalID(id)
            }
            ArithmeticExpr::GlobalID(id) => {
                for (old, new) in mapping.iter() {
                    if id == *old {
                        return ArithmeticExpr::GlobalID(new.clone());
                    }
                }
                ArithmeticExpr::GlobalID(id)
            }
            ArithmeticExpr::Aop(op, a1, a2) => {
                let new_a1 = self.remap_vars_arithmetic(*a1, mapping.clone());
                let new_a2 = self.remap_vars_arithmetic(*a2, mapping.clone());
                ArithmeticExpr::Aop(op, Box::new(new_a1), Box::new(new_a2))
            }
            ArithmeticExpr::Call(fn_id, args) => {
                let mut new_args = vec![];
                for arg in args {
                    new_args.push(self.remap_vars_arithmetic(arg, mapping.clone()));
                }
                ArithmeticExpr::Call(fn_id, new_args)
            }
        }
    }

    fn remap_vars_bool(
        &mut self,
        expr: BooleanExpr,
        mapping: HashMap<String, String>,
    ) -> BooleanExpr {
        match expr {
            BooleanExpr::Bop(op, a1, a2) => {
                let new_a1 = self.remap_vars_arithmetic(a1, mapping.clone());
                let new_a2 = self.remap_vars_arithmetic(a2, mapping.clone());
                BooleanExpr::Bop(op, new_a1, new_a2)
            }
            BooleanExpr::Lop(op, b1, b2) => {
                let new_b1 = self.remap_vars_bool(*b1, mapping.clone());
                let new_b2 = self.remap_vars_bool(*b2, mapping.clone());
                BooleanExpr::Lop(op, Box::new(new_b1), Box::new(new_b2))
            }
            BooleanExpr::True => BooleanExpr::True,
            BooleanExpr::False => BooleanExpr::False,
        }
    }
}

#[test]
fn main() {
    let tokens = match fs::read_to_string("programs/while_func.rimp") {
        Ok(program) => tokenise(program.clone().as_str()),
        Err(e) => {
            println!("Failed to read file. {}", e);
            return;
        }
    };
    println!("{:?}", tokens);
    let result = parse_rimp(tokens);
    println!("{:?}", result);
    match result {
        Ok(program) => {
            let program = program.iter().next().unwrap();
            let mut interpreter = Evaluator::new(program.clone());
            interpreter.interpret_all();
        }
        Err(e) => {
            println!("Failed to parse program. {:?}", e);
        }
    }
}

#[test]
fn test_interpret_step() {
    let tokens = match fs::read_to_string("programs/if.rimp") {
        Ok(program) => tokenise(program.clone().as_str()),
        Err(e) => {
            println!("Failed to read file. {}", e);
            return;
        }
    };
    println!("{:?}", tokens);
    let result = parse_rimp(tokens);
    println!("{:?}", result);
    match result {
        Ok(program) => {
            let program = program.iter().next().unwrap();
            let mut interpreter = Evaluator::new(program.clone());
            interpreter.interpret_step();
            while interpreter.program.len() > 0 {
                println!("{:?}", interpreter);
                interpreter.interpret_step();
            }
        }
        Err(e) => {
            println!("Failed to parse program. {:?}", e);
        }
    }
}

#[test]
fn test_eval_small_step_arithmetic() {
    let tokens = match fs::read_to_string("programs/if.rimp") {
        Ok(program) => tokenise(program.clone().as_str()),
        Err(e) => {
            println!("Failed to read file. {}", e);
            return;
        }
    };
    println!("{:?}", tokens);
    let result = parse_rimp(tokens);
    println!("{:?}", result);
    match result {
        Ok(program) => {
            let program = program.iter().next().unwrap();
            let mut interpreter = Evaluator::new(program.clone());
            interpreter.interpret_step();
            println!("{:?}", interpreter);
            interpreter.eval_back_step_block();
            println!("{:?}", interpreter);
            interpreter.interpret_step();
            println!("{:?}", interpreter);
            interpreter.interpret_step();
            println!("{:?}", interpreter);
            interpreter.eval_back_step_block();
            println!("{:?}", interpreter);
        }
        Err(e) => {
            println!("Failed to parse program. {:?}", e);
        }
    };
}

#[test]
fn print_program() {
    let tokens = match fs::read_to_string("programs/while_func.rimp") {
        Ok(program) => tokenise(program.clone().as_str()),
        Err(e) => {
            println!("Failed to read file. {}", e);
            return;
        }
    };
    println!("{:?}", tokens);
    let result = parse_rimp(tokens);
    println!("{:?}", result);
    match result {
        Ok(program) => {
            let program = program.iter().next().unwrap();
            let mut interpreter = Evaluator::new(program.clone());
            println!("{}", interpreter.get_program());
        }
        Err(e) => {
            println!("Failed to parse program. {:?}", e);
        }
    }
}
