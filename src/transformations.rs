use crate::lexer::Token;
use crate::parser::{ArithmeticExpr, Block, BooleanExpr, FnArgs, ParseResult, RIMPProgram, Stmt};


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
//
//     is_tail_recursive
//
// }

fn function_token_to_string(function_id: &Token) -> String {
    let function_string = match function_id {
        Token::FnId(id) => id.to_string(),
        _ => panic!("Expected an identifier for function id"),
    };
    function_string
}

fn monitor_function_calls(
    program: ParseResult<RIMPProgram, Vec<Token>>,
) -> ParseResult<RIMPProgram, Vec<Token>> {
    todo!()
}

fn while_loop_transform(while_statement: Stmt, counter: i64) -> Block {
    match while_statement {
        Stmt::While(condition, block) => {
            let mut new_block = block.clone();
            let new_condition = condition.clone();
            let mut new_counter = format!("while_loop_{}", counter);
            let assignment = Stmt::Assign(new_counter.clone(), ArithmeticExpr::Num(0));
            new_block.push(Stmt::Assign(
                new_counter.clone(),
                ArithmeticExpr::Aop(
                    new_counter,
                    Box::new(ArithmeticExpr::LocalID(new_counter.clone())),
                    Box::new(ArithmeticExpr::Num(1)),
                ),
            ));
            let new_stmt = Stmt::WhileAcc(new_condition, new_block, new_counter.clone());
            vec![assignment, new_stmt]
        }
        _ => panic!("Expected a while statement"),
    }
}

fn check_if_statement(if_statement: Stmt) -> bool {
    match if_statement {
        Stmt::If(condition, then_block, else_block) => {
            match condition {
                BooleanExpr::Bop(op, b1, b2) => true,
                BooleanExpr::True => {}
                BooleanExpr::False => {}
                BooleanExpr::Lop(_, _, _) => {}
            }
        },
        _ => false,
    }
}

fn extract_variables_used(block: Block) -> Vec<String> {
    let mut variables = vec![];
    for stmt in block {
        match stmt {
            Stmt::Assign(id, expr) => {
                match expr {
                    ArithmeticExpr::Aop(_, a1, a2) => {
                        variables.push(id);
                    }
                    ArithmeticExpr::Num(_) => {}
                    ArithmeticExpr::LocalID(var) => {
                        variables.push(var);
                    }
                    ArithmeticExpr::GlobalID(var) => variables.push(var),
                    ArithmeticExpr::Call(_function_id, params) => {
                        for param in params {
                            todo!("create a arithmetic expression extractor")
                        }
                    }
                }
            }
            Stmt::If(condition, then_block, else_block) => {
                let mut then_variables = extract_variables_used(then_block);
                let mut else_variables = extract_variables_used(else_block);
                variables.append(&mut then_variables);
                variables.append(&mut else_variables);
            }
            Stmt::While(condition, block) => {
                let mut while_variables = extract_variables_used(block);
                variables.append(&mut while_variables);
            }
            Stmt::WhileAcc(condition, block, counter) => {
                let mut while_variables = extract_variables_used(block);
                variables.append(&mut while_variables);
            }
            Stmt::Function(_, _, _) => {}
            Stmt::Skip => {}
            Stmt::Unassign(_, _) => {}
        }
    }
    variables
}