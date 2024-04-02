use crate::parser::{ArithmeticExpr, Block, Stmt};

/// Converts a stmt that is a tail recursive function to
/// a while loop.
/// This function is called when there
///
///
///
fn function_to_loop() {}

// fn check_tail_recursion(function: Stmt) -> bool {
//     let mut is_tail_recursive = false;
//     let Stmt::Function(_, _, body) = function;
//     match body.last().unwrap() {
//         Stmt::Skip => is_tail_recursive = false,
//         Stmt::Assign(_, _) => is_tail_recursive = false,
//         Stmt::If(_, then_branch, else_then) => {
//
//         }
//         Stmt::While(_, _) => {}
//         Stmt::Function(_, _, _) => {}
//     }
//     is_tail_recursive
// }

fn check_call_to_function(function_name: String, stmt: Stmt) {
    match stmt {
        Stmt::Assign(_, value) => match value {
            ArithmeticExpr::Call(fn_name, _args) => {
                if fn_name == function_name {
                    function_to_loop();
                }
            }
            _ => {}
        },
        _ => {}
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
        Stmt::While(condition, block) => unsafe {
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
            let new_stmt = Stmt::While(new_condition, new_block);
            vec![assignment, new_stmt]
        },
        _ => panic!("Expected a while statement"),
    }
}
