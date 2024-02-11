use crate::parser::{ArithmeticExpr, Block, BooleanExpr, Stmt};
use std::collections::{HashMap, HashSet};

type GlobalVar = HashSet<String>; // Set of global variables
type FunctionVars = HashMap<String, Vec<(String)>>; // Map of function names to the local variables used in the function

#[derive(Debug)]
pub struct TypeChecker {
    pub global: GlobalVar,
    pub functions: FunctionVars,
    pub temp_vars: HashMap<String, String>,
    pub temp_then_vars: HashMap<String, String>,
    pub temp_else_vars: HashMap<String, String>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            global: HashSet::new(),
            functions: HashMap::new(),
            temp_vars: HashMap::new(),
            temp_then_vars: HashMap::new(),
            temp_else_vars: HashMap::new(),
        }
    }

    pub fn check(&mut self, program: Block, fn_name: String) -> Vec<String> {
        let mut variables = vec![];
        for stmt in program {
            match stmt {
                Stmt::Skip => {}
                Stmt::Assign(id, value) => {
                    if fn_name == "main" {
                        self.global.insert(id.clone());
                        variables.push(id.clone());
                    } else {
                        if let Some(vars) = self.functions.get_mut(&fn_name) {
                            vars.push(id.clone());
                            variables.push(id.clone());
                        }
                    }
                }
                Stmt::If(boolean, then_block, else_block) => {
                    let mut boolean_vars = self.check_boolean_expr(boolean, fn_name.clone());
                    let mut then_vars = self.check(then_block, fn_name.clone() );
                    let mut else_vars = self.check(else_block, fn_name.clone());

                }
                Stmt::While(_, _) => {}
                Stmt::WhileAcc(_, _, _) => {}
                Stmt::Function(fn_name, arg, block) => {
                    self.functions.insert(fn_name.clone(), arg);
                    let mut function_var = self.check(block, fn_name);
                    variables.append(&mut function_var);
                }
                _ => {}
            }
        }
        variables
    }

    pub fn check_boolean_expr(&mut self, boolean: BooleanExpr, fn_name: String) -> Vec<String> {
        let mut variables = vec![];
        match boolean {
            BooleanExpr::Bop(_, a1, a2) => {
                let mut a1 = self.check_arithmetic_expr(a1, fn_name.clone());
                let mut a2 = self.check_arithmetic_expr(a2, fn_name.clone());
                variables.append(&mut a1);
                variables.append(&mut a2);
            }
            BooleanExpr::Lop(_, b1, b2) => {
                let mut b1 = self.check_boolean_expr(*b1, fn_name.clone());
                let mut b2 = self.check_boolean_expr(*b2, fn_name.clone());
                variables.append(&mut b1);
                variables.append(&mut b2);
            }
            BooleanExpr::True => {}
            BooleanExpr::False => {}
        }
        variables
    }

    pub fn check_arithmetic_expr(&mut self, arithmetic: ArithmeticExpr, fn_name: String) -> Vec<String> {
        let mut variables = vec![];
        match arithmetic {
            ArithmeticExpr::Num(_) => {}
            ArithmeticExpr::LocalID(var_name) => {
                variables.push(var_name);
            }
            ArithmeticExpr::GlobalID(var) => {
                variables.push(var);
            }
            ArithmeticExpr::Aop(_, a1, a2) => {
                let mut a1_vars = self.check_arithmetic_expr(*a1, fn_name.clone());
                let mut a2_vars = self.check_arithmetic_expr(*a2, fn_name.clone());
                variables.append(&mut a1_vars);
                variables.append(&mut a2_vars);
            }
            ArithmeticExpr::Call(function_name, args) => {
                for arg in args {
                    let mut arg_vars = self.check_arithmetic_expr(arg, fn_name.clone());
                    variables.append(&mut arg_vars);
                }
            }
        }
        variables
    }

    // check for overlapping variables and remap them
    pub(crate) fn check_overlapping_vars(&mut self, vars: Vec<String>, then_vars: Vec<String>, else_vars: Vec<String>) -> Vec<String> {
        let mut overlapping_vars = vec![];
        for var in vars {
            if then_vars.contains(&var) || else_vars.contains(&var) {
                overlapping_vars.push(var);
            }
        }
        overlapping_vars
    }
}

#[test]
fn boolean_expr_test() {
    let mut type_checker = TypeChecker::new();
    let boolean = BooleanExpr::Bop("==".to_string(), ArithmeticExpr::Num(1), ArithmeticExpr::LocalID("x".to_string()));
    let else_block = vec![Stmt::Assign("x".to_string(), ArithmeticExpr::Num(0))];
    let vars = type_checker.check_boolean_expr(boolean, "main".to_string());
    let else_vars = type_checker.check(else_block, "main".to_string());
    println!("{:?}", vars);
    println!("{:?}", else_vars);
    let overlapping_vars = type_checker.check_overlapping_vars(vars, vec![], else_vars);
}