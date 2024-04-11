use std::collections::HashMap;
use crate::parser::{ArithmeticExpr, Block, BooleanExpr, Stmt};
use crate::validator::TypeChecker;

/// Converts a stmt that is a tail recursive function to
/// a while loop.
/// This function is called when there
///
///
///
fn function_to_loop(function_name: String, body: Block, args: Vec<String>) -> Block {
    let mut new_block = vec![];
    
    for arg in args {
        new_block.push(Stmt::Assign(arg.clone(), ArithmeticExpr::Num(0)));
    }
    
    // Identify the base case
    let mut base_case = BooleanExpr::True;
    for stmt in body.clone() {
        match stmt {
            Stmt::If(boolean, then_block, else_block) => {
                if then_block.is_empty() && else_block.is_empty() {
                    base_case = boolean.clone();
                }
            }
            _ => {}
        }
    }
    
    
    
    new_block
}



//Ensure that the recursive call is the last operation.
// Make sure there are no subsequent operations that need to be performed after the recursive call.
// If there are, then the function is not tail recursive.
fn check_tail_recursion(function_name: String, body: Block) -> (bool, bool) {
    let mut is_tail_recursive = false;
    let mut recursive_call = false;
    
    for (i, stmt) in body.iter().enumerate() {
        let last_stmt = i == body.len() - 1;
        let (is_tail_recursive_stmt, recursive_call_stmt) = check_tail_recursion_stmt(function_name.clone(), stmt.clone(), last_stmt);
        if is_tail_recursive_stmt {
            is_tail_recursive = true;
            println!("Tail recursive");
            println!("{:?}", stmt);
        }
        if recursive_call_stmt {
            recursive_call = true;
        }
    }

    (is_tail_recursive, recursive_call)
}

fn check_tail_recursion_stmt(function_name: String, stmt: Stmt, last_stmt: bool) -> (bool, bool) {
    let mut is_tail_recursive = false;
    let mut recursive_call = false;

    match stmt {
        Stmt::Assign(_, value) => {
            (is_tail_recursive, recursive_call) = check_tail_recursion_arithmetic(function_name.clone(), value.clone());
            is_tail_recursive = false;
        }
        Stmt::If(_, then_branch, else_branch) => {
            let (is_tail_recursive_then, recursive_call_then) = check_tail_recursion(function_name.clone(), then_branch.clone());
            let (is_tail_recursive_else, recursive_call_else) = check_tail_recursion(function_name.clone(), else_branch.clone());
            if is_tail_recursive_then && is_tail_recursive_else && last_stmt {
                is_tail_recursive = true;
                recursive_call = true;
            } 
            if recursive_call_then || recursive_call_else {
                recursive_call = true;
            }
        }
        Stmt::While(_, block, _) => {
            (is_tail_recursive, recursive_call) = check_tail_recursion(function_name.clone(), block.clone());
        }
        Stmt::Function(_, _, body) => {
            (is_tail_recursive, recursive_call) = check_tail_recursion(function_name.clone(), body.clone());
            if is_tail_recursive && !last_stmt {
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

    (is_tail_recursive, recursive_call)
    
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
    use crate::transformations::check_tail_recursion;
    use crate::parser::{ArithmeticExpr, Block, BooleanExpr, Stmt};

    #[test]
    fn test_check_tail_recursion() {
        let function_name = "f".to_string();
        let body = vec![
            Stmt::Assign("x".to_string(), ArithmeticExpr::Call("f".to_string(), vec![])),
            Stmt::Assign("y".to_string(), ArithmeticExpr::Num(1)),
        ];
        let (is_tail_recursive, recursive_call) = check_tail_recursion(function_name.clone(), body.clone());
        assert_eq!(is_tail_recursive, false); // Not tail recursive as there is an assignment after the recursive call
        assert_eq!(recursive_call, true);
    }
    
    #[test]
    fn test_check_tail_recursion_2() {
        let function_name = "f".to_string();
        let body = vec![
            Stmt::Assign("x".to_string(), ArithmeticExpr::Call("f".to_string(), vec![])),
            Stmt::Return(ArithmeticExpr::Call("f".to_string(), vec![])),
        ];
        let (is_tail_recursive, recursive_call) = check_tail_recursion(function_name.clone(), body.clone());
        assert_eq!(is_tail_recursive, true); // Tail recursive as the recursive call is the last operation
        assert_eq!(recursive_call, true);
    }
    
    #[test]
    fn test_check_tail_recursion_3() {
        let function_name = "f".to_string();
        let body = vec![
            Stmt::Assign("x".to_string(), ArithmeticExpr::Call("f".to_string(), vec![])),
            Stmt::If(
                BooleanExpr::Bop(
                    "==".to_string(),
                    ArithmeticExpr::LocalID("x".to_string()),
                    ArithmeticExpr::Num(0),
                ),
                vec![Stmt::Return(ArithmeticExpr::Call("f".to_string(), vec![]))],
                vec![Stmt::Return(ArithmeticExpr::Num(1))],
            ),
        ];
        let (is_tail_recursive, recursive_call) = check_tail_recursion(function_name.clone(), body.clone());
        assert_eq!(is_tail_recursive, false); // Not tail recursive as there is an if statement after the recursive call
        assert_eq!(recursive_call, true);
    }
    
    #[test]
    fn test_check_tail_recursion_4() {
        let function_name = "f".to_string();
        let body = vec![
            Stmt::Assign("x".to_string(), ArithmeticExpr::Call("f".to_string(), vec![])),
            Stmt::While(
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
            ),
        ];
        let (is_tail_recursive, recursive_call) = check_tail_recursion(function_name.clone(), body.clone());
        assert_eq!(is_tail_recursive, true); // Not tail recursive as there is a while loop after the recursive call
        assert_eq!(recursive_call, true);
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
        let (is_tail_recursive, recursive_call) = check_tail_recursion(function_name.clone(), body.clone());
        assert_eq!(is_tail_recursive, false); // Not tail recursive as there is a while loop after the recursive call
        assert_eq!(recursive_call, true);
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
        Stmt::While(condition, block, acc) => unsafe {
            COUNTER.increment();
            let mut new_block = block.clone();
            let new_condition = condition.clone();
            let mut new_counter = format!("while_loop_{}", COUNTER.value);
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
            let new_stmt = Stmt::WhileR(new_condition, new_block, acc_bool.clone());
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
    }
}