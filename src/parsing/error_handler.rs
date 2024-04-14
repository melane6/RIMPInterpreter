// The error handler module contains the error handling logic for the RIMPInterpreter.
// The parser and lexer modules can return errors such as syntax errors, and semantic errors.
// Parser Error come in the form of a ParseError, and Lexer errors come in the form of a LexError.
// To correctly display ParseErrors:
//     - Use the usize field to determine the token of the error
//     - Traverse the token stream to find the line and column of the error
//     - Display the line and column of the error to the user to indicate where the error is
//     - Display an appropriate error message for the ParseErrorMessage obtained from the ParseError
// To correctly display LexErrors:
//     - Lexing errors are generic and can be displayed as is
//     - Highlight all text that was run to indicate the error
//
// E.g.
//    For the ParseError:
//   ParseError {
//      token: 5,
//      message: ParseErrorMessage::ExpectedToken(Token::FnId("fn".to_string())), E


use crate::interpreter::evaluator::Evaluator;
use crate::lexing::rimp_tokeniser::{Token, tokenise};
use crate::parsing::parser::{Block, parse_rimp};


pub(crate) fn get_program(block: Block) -> String {
    let mut program = String::new();
    // check if it is the last statement to not included ;
    for i in 0..block.len() {
        if i == block.len() - 1 {
            program.push_str(format!("{}", block[i].to_string()).as_str());
        } else {
            program.push_str(format!("{};\n", block[i].to_string()).as_str());
        }
    }
    program
}
pub fn source_point_of_error(tokens: Vec<Token>, parsed: (Block, Vec<Token>)) -> String {
    let (block, not_processed_tokens) = parsed;
    let parsed = get_program(block);
    println!("Parsed: {}", parsed);
    // Remove tokens that are already parsed
    let mut tokens = tokens.clone();
    while tokens.len() > not_processed_tokens.len() {
        let token = tokens.remove(0);
        println!("Token: {:?}", token);
    }
    
    println!("Tokens left: {:?}", tokens);
    
    println!("Error at: {:?}", tokens.last().unwrap());
    let parser_error = parse_rimp(tokens.clone());
    match parser_error {
        Ok(_) => {
            println!("No error found");
        }
        Err(e) => {
            println!("Error found: {:?}", e);
        }
    }
    
    parsed
}
pub fn run(program: String) -> String {
    let tokens = tokenise(program.as_str());
    let tokens = match tokens {
        Ok(tokens) => tokens,
        Err(e) => {
            return format!("Failed to tokenise program due to {:?}", e);
        }
    };
    let result = parse_rimp(tokens);
    
    match result {
        Ok(program) => {
            let program = program.iter().next().unwrap();
            let mut evaluator = Evaluator::new(program.clone());
            evaluator.interpret_all();
            return format!("{:?}", evaluator);
        }
        Err(e) => {
            return format!("Failed to translate program due to {:?}", e);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_point_of_error() {
        let program = "x:=4; y:=7; if 3>y then x=4 else y :=4".to_string();
        println!("{}", run(program));
    }
}