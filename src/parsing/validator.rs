use crate::parsing::parser::{ArithmeticExpr, Block, BooleanExpr, Stmt};
use std::collections::{HashMap, HashSet};

type GlobalVar = HashSet<String>; // Set of global variables
type FunctionVars = HashMap<String, Vec<String>>; // Map of function names to the local variables used in the function

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

    pub fn check(&mut self, program: Block) -> Vec<String> {
        let mut variables = vec![];
        for stmt in program {
            match stmt {
                Stmt::Skip => {}
                Stmt::Assign(id, _value) => {
                    variables.push(id);
                }
                Stmt::If(boolean, then_block, else_block) => {
                    let mut boolean_vars = self.check_boolean_expr(boolean);
                    let mut then_vars = self.check(then_block);
                    let mut else_vars = self.check(else_block);
                    variables.append(&mut boolean_vars);
                    variables.append(&mut then_vars);
                    variables.append(&mut else_vars);
                }
                Stmt::While(boolean, block, _) => {
                    let mut boolean_vars = self.check_boolean_expr(boolean);
                    let mut block_vars = self.check(block);
                    variables.append(&mut boolean_vars);
                    variables.append(&mut block_vars);
                }
                Stmt::WhileR(_, _, _) => {}
                Stmt::Function(_fn_name, _arg, block) => {
                    let mut function_var = self.check(block);
                    variables.append(&mut function_var);
                }
                _ => {}
            }
        }
        variables
    }

    pub fn check_boolean_expr(&mut self, boolean: BooleanExpr) -> Vec<String> {
        let mut variables = vec![];
        match boolean {
            BooleanExpr::Bop(_, a1, a2) => {
                let mut a1 = self.check_arithmetic_expr(a1);
                let mut a2 = self.check_arithmetic_expr(a2);
                variables.append(&mut a1);
                variables.append(&mut a2);
            }
            BooleanExpr::Lop(_, b1, b2) => {
                let mut b1 = self.check_boolean_expr(*b1);
                let mut b2 = self.check_boolean_expr(*b2);
                variables.append(&mut b1);
                variables.append(&mut b2);
            }
            BooleanExpr::True => {}
            BooleanExpr::False => {}
            BooleanExpr::Not(b1) => {
                let mut b1 = self.check_boolean_expr(*b1);
                variables.append(&mut b1);
            }
        }
        variables
    }

    pub fn check_arithmetic_expr(
        &mut self,
        arithmetic: ArithmeticExpr,
    ) -> Vec<String> {
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
                let mut a1_vars = self.check_arithmetic_expr(*a1);
                let mut a2_vars = self.check_arithmetic_expr(*a2);
                variables.append(&mut a1_vars);
                variables.append(&mut a2_vars);
            }
            ArithmeticExpr::Call(_function_name, args) => {
                for arg in args {
                    let mut arg_vars = self.check_arithmetic_expr(arg);
                    variables.append(&mut arg_vars);
                }
            }
        }
        variables
    }

    // check for overlapping variables and remap them
    pub(crate) fn check_overlapping_vars(
        &mut self,
        vars: Vec<String>,
        then_vars: Vec<String>,
        else_vars: Vec<String>,
    ) -> Vec<String> {
        let mut overlapping_vars = vec![];
        for var in vars {
            if then_vars.contains(&var) || else_vars.contains(&var) {
                overlapping_vars.push(var);
            }
        }
        overlapping_vars
    }
}

