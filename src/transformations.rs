
use crate::lexer::Token;
use crate::parser::{Block, FnArgs, RIMPProgram, Stmt, ParseResult};

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

fn monitor_function_calls(program: ParseResult<RIMPProgram, Vec<Token>>) -> ParseResult<RIMPProgram, Vec<Token>> {
    todo!()
}