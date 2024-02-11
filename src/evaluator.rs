use crate::parser::{ArithmeticExpr, Block, BooleanExpr, RIMPProgram, Stmt, parse_rimp};
use crate::validator::TypeChecker;
use std::collections::{HashMap, HashSet};
use std::fs;
use crate::lexer::tokenise;

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
    functions: HashMap<String, (Environment, Block)>,
    function_name: Vec<String>,
}

impl RIMPInterpreter {
    fn new() -> RIMPInterpreter {
        RIMPInterpreter {
            counter: Counter::new(),
            environment: Environment::new(),
            functions: Default::default(),
            function_name: vec!["main".to_string()],
        }
    }

    fn interpret_all(&mut self, program: HashSet<Block>)  {
        let block = program.iter().next().unwrap();
        self.eval_block(block.clone());
        println!("{:?}", self.environment);
        println!("{:?}", self.functions);
    }

    fn interpret_step(&mut self, program: RIMPProgram) {
        match program {
            RIMPProgram::Program(block) => {
                self.eval_block(block);
            }
        }
        // todo!("Implement the interpret step function")
    }

    fn eval_arithmetic_expr(&mut self, expr: ArithmeticExpr) -> i64 {
        match expr {
            ArithmeticExpr::Num(n) => n,
            ArithmeticExpr::LocalID(id) => {
                if self.function_name.last().unwrap() == "main" {
                    println!("{:?}", self.environment);
                    println!("{}", id.clone());
                    self.environment.get(&id).unwrap().clone()
                } else {
                    let function_env = self
                        .functions
                        .get(self.function_name.last().unwrap())
                        .unwrap()
                        .0
                        .clone();
                    function_env.get(&id).unwrap().clone()
                }
            }
            ArithmeticExpr::GlobalID(id) => self.environment.get(&id).unwrap().clone(),
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
                    let value = self.eval_arithmetic_expr(args[0].clone());
                    self.functions
                        .get_mut(self.function_name.last().unwrap())
                        .unwrap()
                        .0
                        .insert(arg.0, value);
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
            self.environment.get("return").unwrap().clone()
        } else {
            self.functions.get(&function_name).unwrap().0.get("return").unwrap().clone()
        }
    }

    fn get_function_env(&mut self, function_name: String) -> Environment {
        if function_name == "main" {
            self.environment.clone()
        } else {
            self.functions.get(&function_name).unwrap().0.clone()
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

    fn eval_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Assign(id, expr) => {
                if self.function_name.last().unwrap() == "main" {
                    let value = self.eval_arithmetic_expr(expr);
                    self.environment.insert(id, value);
                } else {
                    let value = self.eval_arithmetic_expr(expr);
                    self.functions
                        .get_mut(self.function_name.last().unwrap())
                        .unwrap()
                        .0
                        .insert(id, value);
                }
            }
            Stmt::Skip => {}
            Stmt::Unassign(id, value) => {
                todo!("Implement the unassign statement")
            }
            Stmt::If(boolean_expr, mut then_block, mut else_block) => {
                let mut validator = TypeChecker::new();
                // get environment name
                let env_name = self.function_name.last().unwrap();
                let condition_vars = validator.check_boolean_expr(
                    boolean_expr.clone(),
                    env_name.clone(),
                );
                // check if the condition variables are in the environment
                let then_vars = validator.check(then_block.clone(), env_name.clone());
                let else_vars = validator.check(else_block.clone(), env_name.clone());
                let overlapping_vars =
                    validator.check_overlapping_vars(condition_vars, then_vars, else_vars);
                if !overlapping_vars.is_empty() {
                    let mut mapping = HashMap::new();
                    for var in overlapping_vars {
                        // copy the value of the variable to the environment
                        if self.function_name.last().unwrap() == "main" {
                            let value = self.environment.get(&var).unwrap().clone();
                            let new_var = format!("{}'", var);
                            mapping.insert(var, new_var.clone());
                            self.environment.insert(new_var, value);
                        } else {
                            let value = self
                                .functions
                                .get(self.function_name.last().unwrap())
                                .unwrap()
                                .0
                                .get(&var)
                                .unwrap()
                                .clone();
                            let new_var = format!("{}'", var);
                            mapping.insert(var, new_var.clone());
                            self.functions
                                .get_mut(self.function_name.last().unwrap())
                                .unwrap()
                                .0
                                .insert(new_var, value);
                        }
                    }
                    // remap all references to the variable in the then and else block
                    then_block = self.remap_vars(then_block, mapping.clone());
                    else_block = self.remap_vars(else_block, mapping.clone());
                }
                if self.eval_boolean_expr(boolean_expr) {
                    self.eval_block(then_block);
                } else {
                    self.eval_block(else_block);
                }
            }
            Stmt::While(boolean, block) => {
                while self.eval_boolean_expr(boolean.clone()) {
                    self.eval_block(block.clone());
                }
                println!("this was in the middle{:?}", self.environment);
            }
            Stmt::Function(function_name, args, block) => {
                let mut function_env = Environment::new();
                for arg in args {
                    function_env.insert(arg, 0); // default value
                }
                function_env.insert("return".to_string(), 0);
                self.functions.insert(function_name, (function_env, block));
            }
            _ => {}
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
    let mut interpreter = RIMPInterpreter::new();
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
            interpreter.interpret_all(program);
        }
        Err(e) => {
            println!("Failed to parse program. {:?}", e);
        }
    }
}
