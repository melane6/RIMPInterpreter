use std::collections::HashMap;
use crate::parser::{ArithmeticExpr, Block, BooleanExpr, Stmt};
use crate::validator::TypeChecker;

/// Converts a stmt that is a tail recursive function to
/// a while loop.
/// This function is called when there
///
///
///
pub(crate) fn function_to_loop(function_name: String, mut body: Block, args: Vec<String>) -> Stmt {
    let mut new_block = vec![];
    let mut validation = TypeChecker::new();
    
    // Identify the base case
    let (is_tail_recursive, recursive_call, base_case_stmt) = check_tail_recursion(function_name.clone(), body.clone());
    
    let mut base_case = BooleanExpr::False;
    let mut base_case_var = vec![];
    
    
    if is_tail_recursive {
        match base_case_stmt.clone() {
            Stmt::If(bool, _, _) => {
                base_case = bool.clone();
                base_case_var = validation.check_boolean_expr(bool.clone());
            }
            Stmt::While(bool, _, _) => {
                base_case = bool.clone();
                base_case_var = validation.check_boolean_expr(bool.clone());
            }
            _ => {}
        }
        
        // Remove the last statement from the body
        body.remove(body.len() - 1);

        // If the function is tail recursive, we need to add the base case
        // to the while loop
        
        for stmt in body {
            new_block.push(stmt);
        }
        
        
        match base_case_stmt { 
            Stmt::If(_, then_block, else_block) => {
                let mut new_then_block = then_block.clone();
                let mut new_else_block = else_block.clone();
                
                // Identify in which block the recursion occurs
                let (_, recursive_call_then, _) = check_tail_recursion(function_name.clone(), then_block.clone());
                let (_, recursive_call_else, _) = check_tail_recursion(function_name.clone(), else_block.clone());
                
                if recursive_call_else {
                    base_case = BooleanExpr::Not(Box::new(base_case.clone()));
                }
                
                new_then_block.remove(new_then_block.len() - 1);
                let mut while_loop_block = while_loop_transform(Stmt::While(base_case.clone(), new_then_block, BooleanExpr::False));
                new_block.append(&mut while_loop_block);
                for stmt in new_else_block {
                    match stmt { 
                        Stmt::Return(a1) => match a1.clone() { 
                            ArithmeticExpr::Call(fn_name, args_call) => {
                                if fn_name != function_name {
                                    new_block.push(Stmt::Assign("return".to_string(), a1.clone()));
                                } else { 
                                    for i in 0..args.len() {
                                        new_block.push(Stmt::Assign(args[i].clone(), args_call[i].clone()));
                                    }
                                }
                            }
                            _ => {
                                new_block.push(Stmt::Assign("return".to_string(), a1.clone()));
                            }
                        }
                        _ => {
                            new_block.push(stmt);
                        }
                    }
                }
            }
            Stmt::While(_,mut  block, _) => {
                block.remove(block.len() - 1);
                new_block.append(&mut block);
                new_block.push(Stmt::While(base_case, block, BooleanExpr::False));
            }
            _ => {}
        }
    } else {
        for stmt in body {
            new_block.push(stmt);
        }
    }
    
    Stmt::Function(function_name, args, new_block)
}




//Ensure that the recursive call is the last operation.
// Make sure there are no subsequent operations that need to be performed after the recursive call.
// If there are, then the function is not tail recursive.
fn check_tail_recursion(function_name: String, body: Block) -> (bool, bool, Stmt) {
    let mut is_tail_recursive = false;
    let mut recursive_call = false;
    let mut base_case = Stmt::Skip;
    
    for (i, stmt) in body.iter().enumerate() {
        let last_stmt = i == body.len() - 1;
        let (is_tail_recursive_stmt, recursive_call_stmt, base_case_stmt) = check_tail_recursion_stmt(function_name.clone(), stmt.clone(), last_stmt);
        if is_tail_recursive_stmt {
            is_tail_recursive = true;
            base_case = base_case_stmt.clone();
        }
        if recursive_call_stmt {
            recursive_call = true;
        }
    }

    (is_tail_recursive, recursive_call, base_case)
}

fn check_tail_recursion_stmt(function_name: String, stmt: Stmt, last_stmt: bool) -> (bool, bool, Stmt) {
    let mut is_tail_recursive = false;
    let mut recursive_call = false;
    let mut base_case = Stmt::Skip;

    match stmt {
        Stmt::Assign(_, value) => {
            (is_tail_recursive, recursive_call) = check_tail_recursion_arithmetic(function_name.clone(), value.clone());
            is_tail_recursive = false;
        }
        Stmt::If(bool, then_branch, else_branch) => {
            let (is_tail_recursive_then, recursive_call_then, _) = check_tail_recursion(function_name.clone(), then_branch.clone());
            let (is_tail_recursive_else, recursive_call_else, _) = check_tail_recursion(function_name.clone(), else_branch.clone());
            if is_tail_recursive_then || is_tail_recursive_else && last_stmt {
                base_case = Stmt::If(bool, then_branch, else_branch);
                is_tail_recursive = true;
                recursive_call = true;
            } 
            if recursive_call_then || recursive_call_else {
                recursive_call = true;
            }
        }
        Stmt::While(bool, block, acc_bool) => {
            (is_tail_recursive, recursive_call, base_case) = check_tail_recursion(function_name.clone(), block.clone());
            if is_tail_recursive && !last_stmt {
                base_case = Stmt::Skip;
                is_tail_recursive = false;
                recursive_call = true;
            } else if is_tail_recursive && last_stmt {
                base_case = Stmt::While(bool, block, acc_bool);
            }
        }
        Stmt::Function(_, _, body) => {
            (is_tail_recursive, recursive_call, base_case) = check_tail_recursion(function_name.clone(), body.clone());
            if is_tail_recursive && !last_stmt {
                base_case = Stmt::Skip;
                is_tail_recursive = false;
                recursive_call = true;
            }
        }
        Stmt::Return(arithmetic_expr) => {
            (is_tail_recursive, recursive_call) = check_tail_recursion_arithmetic(function_name.clone(), arithmetic_expr.clone());
            if recursive_call && is_tail_recursive {
                is_tail_recursive = true;
            }
        }
        _ => {}
    }

    (is_tail_recursive, recursive_call, base_case)
    
}

fn check_tail_recursion_arithmetic(function_name: String, arithmetic_expr: ArithmeticExpr) -> (bool, bool) {
    let mut is_tail_recursive = false;
    let mut recursive_call = false;
    match arithmetic_expr {
        ArithmeticExpr::Call(fn_name, _args) => {
            if fn_name == function_name {
                is_tail_recursive = true;
                recursive_call = true;
            }
        }
        ArithmeticExpr::Aop(_, a1, a2) => {
            if let ArithmeticExpr::Call(fn_name, _args) = *a1 {
                if fn_name == function_name {
                    recursive_call = true;
                }
            }
            if let ArithmeticExpr::Call(fn_name, _args) = *a2{ 
                if fn_name == function_name {
                    recursive_call = true;
                }
            }
        }
        _ => {}
    }
    (is_tail_recursive, recursive_call)
}

// Test the check_tail_recursion
#[cfg(test)]
mod test {
    use crate::transformations::{check_tail_recursion, function_to_loop};
    use crate::parser::{ArithmeticExpr, Block, BooleanExpr, Stmt};

    #[test]
    fn test_check_tail_recursion() {
        let function_name = "f".to_string();
        let body = vec![
            Stmt::Assign("x".to_string(), ArithmeticExpr::Call("f".to_string(), vec![])),
            Stmt::Assign("y".to_string(), ArithmeticExpr::Num(1)),
        ];
        let (is_tail_recursive, recursive_call, base) = check_tail_recursion(function_name.clone(), body.clone());
        assert_eq!(is_tail_recursive, false); // Not tail recursive as there is an assignment after the recursive call
        assert_eq!(recursive_call, true);
        assert_eq!(base, Stmt::Skip);
    }
    
    #[test]
    fn test_check_tail_recursion_2() {
        let function_name = "f".to_string();
        let body = vec![
            Stmt::Assign("x".to_string(), ArithmeticExpr::Call("f".to_string(), vec![])),
            Stmt::Return(ArithmeticExpr::Call("f".to_string(), vec![])),
        ];
        let (is_tail_recursive, recursive_call, base) = check_tail_recursion(function_name.clone(), body.clone());
        assert_eq!(is_tail_recursive, true); // Tail recursive as the recursive call is the last operation
        assert_eq!(recursive_call, true);
        assert_eq!(base, Stmt::Skip);
    }
    
    #[test]
    fn test_check_tail_recursion_3() {
        let function_name = "f".to_string();
        let if_stmt = Stmt::If(
            BooleanExpr::Bop(
                "==".to_string(),
                ArithmeticExpr::LocalID("x".to_string()),
                ArithmeticExpr::Num(0),
            ),
            vec![Stmt::Return(ArithmeticExpr::Call("f".to_string(), vec![]))],
            vec![Stmt::Return(ArithmeticExpr::Num(1))],
        );
        let body = vec![
            Stmt::Assign("x".to_string(), ArithmeticExpr::Call("f".to_string(), vec![])),
            if_stmt.clone(),
        ];
        let (is_tail_recursive, recursive_call, base) = check_tail_recursion(function_name.clone(), body.clone());
        assert_eq!(is_tail_recursive, true); // Not tail recursive as there is an if statement after the recursive call
        assert_eq!(recursive_call, true);
        assert_eq!(base, if_stmt.clone());
    }
    
    #[test]
    fn test_check_tail_recursion_4() {
        let function_name = "f".to_string();
        let while_loop = Stmt::While(
            BooleanExpr::Bop(
                "==".to_string(),
                ArithmeticExpr::LocalID("x".to_string()),
                ArithmeticExpr::Num(0),
            ),
            vec![Stmt::Return(ArithmeticExpr::Call("f".to_string(), vec![]))],
            BooleanExpr::Bop(
                "==".to_string(),
                ArithmeticExpr::LocalID("x".to_string()),
                ArithmeticExpr::Num(0),
            )
        );
        let body = vec![
            Stmt::Assign("x".to_string(), ArithmeticExpr::Call("f".to_string(), vec![])),
            while_loop.clone(),
        ];
        let (is_tail_recursive, recursive_call, base) = check_tail_recursion(function_name.clone(), body.clone());
        assert_eq!(is_tail_recursive, true); // Not tail recursive as there is a while loop after the recursive call
        assert_eq!(recursive_call, true);
        assert_eq!(base, while_loop.clone());
    }

    #[test]
    fn test_check_tail_recursion_5() {
        let function_name = "f".to_string();
        let body = vec![
            Stmt::Assign("x".to_string(), ArithmeticExpr::Call("f".to_string(), vec![])),
            Stmt::While(
                BooleanExpr::Bop(
                    "==".to_string(),
                    ArithmeticExpr::LocalID("x".to_string()),
                    ArithmeticExpr::Num(0),
                ),
                vec![Stmt::Assign("x".to_string(), ArithmeticExpr::Call("f".to_string(), vec![]))],
                BooleanExpr::Bop(
                    "==".to_string(),
                    ArithmeticExpr::LocalID("x".to_string()),
                    ArithmeticExpr::Num(0),
                )
            ),
        ];
        let (is_tail_recursive, recursive_call, base) = check_tail_recursion(function_name.clone(), body.clone());
        assert_eq!(is_tail_recursive, false); // Not tail recursive as there is a while loop after the recursive call
        assert_eq!(recursive_call, true);
        assert_eq!(base, Stmt::Skip);
    }
    
    #[test]
    fn test_check_tail_recursion_6() {
        let function_name = "f".to_string();
        let if_stmt = Stmt::If(
            BooleanExpr::Bop(
                "==".to_string(),
                ArithmeticExpr::LocalID("x".to_string()),
                ArithmeticExpr::Num(0),
            ),
            vec![Stmt::Return(ArithmeticExpr::Call("f".to_string(), vec![]))],
            vec![Stmt::Return(ArithmeticExpr::Num(1))],
        );
        let body = vec![
            Stmt::Assign("x".to_string(), ArithmeticExpr::Call("f".to_string(), vec![])),
            if_stmt.clone(),
        ];
        let (is_tail_recursive, recursive_call, base) = check_tail_recursion(function_name.clone(), body.clone());
        assert_eq!(is_tail_recursive, true); // Not tail recursive as there is an if statement after the recursive call
        assert_eq!(recursive_call, true);
        assert_eq!(base, if_stmt.clone());
    }

    #[test]
    fn test_function_to_loop() {
        let function_name = "f".to_string();
        let args = vec!["x".to_string()];
        let body = vec![
            Stmt::If(
                BooleanExpr::Bop(
                    "==".to_string(),
                    ArithmeticExpr::LocalID("x".to_string()),
                    ArithmeticExpr::Num(0),
                ),
                vec![Stmt::Return(ArithmeticExpr::Num(0))],
                vec![Stmt::Return(ArithmeticExpr::Call("f".to_string(), vec![
                    ArithmeticExpr::Aop(
                        "-".to_string(),
                        Box::new(ArithmeticExpr::LocalID("x".to_string())),
                        Box::new(ArithmeticExpr::Num(1)),
                    )
                ]))],
            )
        ];
        let new_block = function_to_loop(function_name.clone(), body.clone(), args.clone());
        
        // assert_eq!(new_block, expected_block);
    }
}








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

// Counter for the number of while loops
static mut COUNTER: Counter = Counter { value: 0 };

pub(crate) fn while_loop_transform(while_statement: Stmt) -> Block {
    match while_statement {
        Stmt::While(condition, block, _) => unsafe {
            COUNTER.increment();
            let mut new_block = block.clone();
            let new_condition = condition.clone();
            let new_counter = format!("while_loop_{}", COUNTER.value);
            let assignment = Stmt::Assign(new_counter.clone(), ArithmeticExpr::Num(0));
            new_block.push(Stmt::Assign(
                new_counter.clone(),
                ArithmeticExpr::Aop(
                    "+".to_string(),
                    Box::new(ArithmeticExpr::LocalID(new_counter.clone())),
                    Box::new(ArithmeticExpr::Num(1)),
                ),
            ));
            let acc_bool = BooleanExpr::Bop(
                "==".to_string(),
                ArithmeticExpr::LocalID(new_counter.clone()),
                ArithmeticExpr::Num(0),
            );
            let new_stmt = Stmt::While(new_condition, new_block, acc_bool.clone());
            vec![assignment, new_stmt]
        },
        _ => panic!("Expected a while statement"),
    }
}

pub(crate) fn translate_program(program: Block) -> Block {
    let mut new_program = vec![];
    for stmt in program {
        match stmt {
            Stmt::If(boolean, then_block, else_block) => {
                let overlapping_vars = check_overlapping(boolean.clone(), then_block.clone(), else_block.clone());
                let modified_block = modify_block(overlapping_vars, boolean.clone(), then_block.clone(), else_block.clone());
                for stmt in modified_block {
                    new_program.push(stmt);
                }
            }
            _ => {
                new_program.push(stmt);
            }
        }
    }
    new_program
}

fn check_overlapping(boolean_expr : BooleanExpr ,then_block: Block, else_block: Block) -> Vec<String> {
    let mut validator = TypeChecker::new();
    // get environment name
    let condition_vars = validator.check_boolean_expr(boolean_expr.clone());
    // check if the condition variables are in the environment
    let then_vars = validator.check(then_block.clone());
    let else_vars = validator.check(else_block.clone());
    let overlapping_vars = validator.check_overlapping_vars(condition_vars, then_vars, else_vars);
    overlapping_vars
}

fn modify_block(overlapping_vars: Vec<String>, boolean_expr: BooleanExpr, then_block: Block, else_block: Block) -> Block {
    if !overlapping_vars.is_empty() {
        let mut mapping = HashMap::new();
        let mut addtional_stmts = vec![];
        // Find the overlapping variables and create a new variable with the same value
        for var in overlapping_vars {
            let new_var = format!("{}'", var);
            mapping.insert(new_var.clone(), var);
        }
        let mut bool_expr = boolean_expr.clone();
        let mut then_block = then_block.clone();
        let mut else_block = else_block.clone();

        // remap all references to the variable in the then and else block
        bool_expr = remap_vars_bool(bool_expr, mapping.clone());
        then_block = remap_vars(then_block, mapping.clone());
        else_block = remap_vars(else_block, mapping.clone());

        for (new_var, var) in mapping {
            addtional_stmts.push(Stmt::Assign(new_var.clone(), ArithmeticExpr::LocalID(var.clone())));
        }

        addtional_stmts.push(Stmt::If(bool_expr, then_block, else_block));
        return addtional_stmts
    } else {
        return vec![Stmt::If(boolean_expr, then_block, else_block)]
    }
}

fn remap_vars(block: Vec<Stmt>, mapping: HashMap<String, String>) -> Block {
    let mut new_block = vec![];
    for stmt in block {
        match stmt {
            Stmt::Assign(id, value) => {
                let arith_expr = remap_vars_arithmetic(value, mapping.clone());
                for (new, old) in mapping.iter() {
                    if id == *old {
                        new_block.push(Stmt::Assign(new.clone(), arith_expr.clone()));
                    }
                }
            }
            Stmt::If(boolean, then_block, else_block) => {
                let new_boolean = remap_vars_bool(boolean, mapping.clone());
                let new_then_block = remap_vars(then_block, mapping.clone());
                let new_else_block = remap_vars(else_block, mapping.clone());
                new_block.push(Stmt::If(new_boolean, new_then_block, new_else_block));
            }
            Stmt::While(boolean_expr, block_while, acc) => {
                let new_boolean = remap_vars_bool(boolean_expr, mapping.clone());
                let block = remap_vars(block_while, mapping.clone());
                new_block.push(Stmt::While(new_boolean, block, acc));
            }
            Stmt::Function(function_name, args, fn_block) => {
                let block = remap_vars(fn_block, mapping.clone());
                new_block.push(Stmt::Function(function_name, args, block));
            }
            _ => {}
        }
    }
    new_block
}

fn remap_vars_arithmetic(
    expr: ArithmeticExpr,
    mapping: HashMap<String, String>,
) -> ArithmeticExpr {
    match expr {
        ArithmeticExpr::Num(n) => ArithmeticExpr::Num(n),
        ArithmeticExpr::LocalID(id) => {
            for (new, old) in mapping.iter() {
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
            let new_a1 = remap_vars_arithmetic(*a1, mapping.clone());
            let new_a2 = remap_vars_arithmetic(*a2, mapping.clone());
            ArithmeticExpr::Aop(op, Box::new(new_a1), Box::new(new_a2))
        }
        ArithmeticExpr::Call(fn_id, args) => {
            let mut new_args = vec![];
            for arg in args {
                new_args.push(remap_vars_arithmetic(arg, mapping.clone()));
            }
            ArithmeticExpr::Call(fn_id, new_args)
        }
    }
}

fn remap_vars_bool(
    expr: BooleanExpr,
    mapping: HashMap<String, String>,
) -> BooleanExpr {
    match expr {
        BooleanExpr::Bop(op, a1, a2) => {
            let new_a1 = remap_vars_arithmetic(a1, mapping.clone());
            let new_a2 = remap_vars_arithmetic(a2, mapping.clone());
            BooleanExpr::Bop(op, new_a1, new_a2)
        }
        BooleanExpr::Lop(op, b1, b2) => {
            let new_b1 = remap_vars_bool(*b1, mapping.clone());
            let new_b2 = remap_vars_bool(*b2, mapping.clone());
            BooleanExpr::Lop(op, Box::new(new_b1), Box::new(new_b2))
        }
        BooleanExpr::True => BooleanExpr::True,
        BooleanExpr::False => BooleanExpr::False,
        BooleanExpr::Not(b) => {
            let new_b = remap_vars_bool(*b, mapping.clone());
            BooleanExpr::Not(Box::new(new_b))
        }
    }
}