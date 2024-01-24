use crate::lexer::{Keyword, Operation, Token};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ParseError {
    UnexpectedToken(Token),
    SyntaxError(String),
    FunctionNotImplemented(String),
    ExpectedToken(Token),
}

pub type Block = Vec<Stmt>;

pub type FnArgs = Vec<String>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RIMPProgram {
    Program(Stmts),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stmts {
    Statements(Stmt, Box<Stmts>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stmt {
    Skip,
    Assign(String, ArithmeticExpr),
    Unassign(String, ArithmeticExpr),
    If(BooleanExpr, Block, Block),
    While(BooleanExpr, Block),
    Function(String, FnArgs, Block), // function name, arguments, body
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArithmeticExpr {
    Num(i64),
    Var(String),
    Aop(String, Box<ArithmeticExpr>, Box<ArithmeticExpr>),
    Call(String, Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BooleanExpr {
    True,
    False,
    Bop(String, ArithmeticExpr, ArithmeticExpr),
    Lop(String, Box<BooleanExpr>, Box<BooleanExpr>),
}
type TokenList = Vec<Token>;

struct Parser {
    tokens: TokenList,
    variables: HashSet<String>,              // variable id, value?
    functions: HashMap<String, Vec<String>>, // function id, arguments and variable in that scope
    index: i64,
}

type ParseResult<T> = Result<T, ParseError>;

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            variables: HashSet::new(),
            functions: HashMap::new(),
            index: 0,
        }
    }

    pub fn peek(&mut self) -> Option<&Token> {
        let index = self.index as usize;
        self.tokens.get(index)
    }

    pub fn consume(&mut self) -> Option<&Token> {
        self.increment_index();
        let token = self.peek();
        token
    }

    pub fn increment_index(&mut self) {
        self.index += 1;
    }

    pub fn get_precedence(&mut self, op: &Operation) -> i32 {
        match op {
            Operation::Plus => 1,
            Operation::Minus => 1,
            Operation::Multiply => 2,
            Operation::Divide => 2,
            Operation::Mod => 2,
            Operation::Power => 3,
            _ => 0,
        }
    }

    pub fn is_left_associative(&self, op: &Operation) -> bool {
        match op {
            Operation::Plus => true,
            Operation::Minus => true,
            Operation::Multiply => true,
            Operation::Divide => true,
            Operation::Power => false,
            _ => false,
        }
    }



    pub fn valid_function_name(&mut self, name: String) -> bool {
        self.functions.contains_key(&name)
    }
}

fn parser(token_list: TokenList) -> ParseResult<RIMPProgram> {
    let mut parser_object = Parser::new(token_list);
    let (result, parser_object) = parse_stmts(parser_object);
    match result {
        Ok(stmts) => Ok(RIMPProgram::Program(stmts)),
        Err(e) => Err(e),
    }
}

fn parse_stmts(parser: Parser) -> (ParseResult<Stmts>, Parser) {
    // stmts -> stmt stmts | ε
    let (result, parser) = parse_stmt(parser);
    match result {
        Ok(stmt) => {
            let (result, parser) = parse_stmts(parser);
            match result {
                Ok(stmts) => (Ok(Stmts::Statements(stmt, Box::new(stmts))), parser),
                Err(e) => (Err(e), parser),
            }
        }
        Err(e) => (Err(e), parser),
    }
}

fn parse_stmt(mut parser: Parser) -> (ParseResult<Stmt>, Parser) {
    // stmt -> skip | x := aexp | if bexp then block else block | while bexp do block | Fn
    match parser.peek() {
        Some(Token::Keyword(Keyword::Skip)) => {
            // skip
            parser.consume();
            (Ok(Stmt::Skip), parser)
        }
        Some(Token::Id(x)) => {
            // assignment
            parser.consume();
            match parser.consume() {
                Some(Token::Op(Operation::Assign)) => {
                    let (result, mut parser) = parse_aexp(parser);
                    match result {
                        Ok(aexp) => {
                            parser.variables.insert(x.clone());
                            (Ok(Stmt::Assign(x.clone(), aexp)), parser)
                        }
                        Err(e) => (Err(e), parser),
                    }
                }
                Some(Token::Op(Operation::Unassign)) => {
                    // Need to check that x is assigned
                    if !parser.variables.contains(x) {
                        return (
                            Err(ParseError::SyntaxError(
                                "Variable has not been assigned yet".to_string(),
                            )),
                            parser,
                        );
                    }
                    let (results, new_parser) = parse_aexp(parser);
                    match results {
                        Ok(aexp) => (Ok(Stmt::Unassign(x.clone(), aexp)), new_parser),
                        Err(e) => (Err(e), new_parser),
                    }
                }
                _ => (Err(ParseError::UnexpectedToken(parser.peek().unwrap().clone())), parser)
            }
        }
        Some(Token::Keyword(Keyword::If)) => {
            parser.consume();
            let (result, mut new_parser) = parse_bexp(parser);
            match result {
                Ok(bexp) => match new_parser.consume() {
                    Some(Token::Keyword(Keyword::Then)) => {
                        let (result_block_then, mut parser_then) = parse_block(new_parser);
                        match result_block_then {
                            Ok(block1) => match parser_then.consume() {
                                Some(Token::Keyword(Keyword::Else)) => {
                                    let (result_block_else, parser_else) = parse_block(parser_then);
                                    match result_block_else {
                                        Ok(block2) => {
                                            (Ok(Stmt::If(bexp, block1, block2)), parser_else)
                                        }
                                        Err(e) => (Err(e), parser_else),
                                    }
                                }
                                _ => (
                                    Err(ParseError::UnexpectedToken(
                                        parser_then.peek().unwrap().clone(),
                                    )),
                                    parser_then,
                                ),
                            },
                            _ => (
                                Err(ParseError::UnexpectedToken(
                                    parser_then.peek().unwrap().clone(),
                                )),
                                parser_then,
                            ),
                        }
                    }
                    _ => (
                        Err(ParseError::UnexpectedToken(
                            new_parser.peek().unwrap().clone(),
                        )),
                        new_parser,
                    ),
                },
                _ => (
                    Err(ParseError::UnexpectedToken(
                        new_parser.peek().unwrap().clone(),
                    )),
                    new_parser,
                ),
            }
        }
        Some(Token::Keyword(Keyword::While)) => {
            parser.consume();
            let (result, mut parser_bexp) = parse_bexp(parser);
            match result {
                Ok(bexp) => match parser_bexp.consume() {
                    Some(Token::Keyword(Keyword::Do)) => {
                        let (result_do_block, parse_do) = parse_block(parser_bexp);
                        match result_do_block {
                            Ok(block) => (Ok(Stmt::While(bexp, block)), parse_do),
                            Err(e) => (Err(e), parse_do),
                        }
                    }
                    _ => (
                        Err(ParseError::UnexpectedToken(
                            parser_bexp.peek().unwrap().clone(),
                        )),
                        parser_bexp,
                    ),
                },
                Err(e) => (Err(e), parser_bexp),
            }
        }
        Some(Token::Keyword(Keyword::Fn)) => {
            // fn id (args) block
            parser.consume();
            match parser.consume() {
                Some(Token::FnId(name)) => {
                    let (result_args, mut parser_args) = parse_args(parser, name);
                    match result_args {
                        Ok(args) => {
                            let (result_block, parser_block) = parse_block(parser_args);
                            match result_block {
                                Ok(block) => {
                                    (Ok(Stmt::Function(name.clone(), args, block)), parser_block)
                                }
                                Err(e) => (Err(e), parser_block),
                            }
                        }
                        _ => (
                            Err(ParseError::UnexpectedToken(
                                parser_args.peek().unwrap().clone(),
                            )),
                            parser_args,
                        ),
                    }
                }
                _ => (Err(ParseError::ExpectedToken(Token::FnId(
                    "function_name".to_string(),
                ))), parser)
            }
        }
        _ => (Err(ParseError::UnexpectedToken(parser.peek().unwrap().clone())), parser)
    }
}

fn parse_args(mut parser: Parser, function_name: &String) -> (ParseResult<Vec<String>>, Parser) {
    todo!()
}

fn parse_block(parser: Parser) -> (ParseResult<Block>, Parser) {
    todo!()
}

fn parse_bexp(parser: Parser) -> (ParseResult<BooleanExpr>, Parser) {
    todo!()
}

fn parse_aexp(mut parser: Parser) -> (ParseResult<ArithmeticExpr>, Parser) {
    let mut output = vec![];
    let mut ops_stack: Vec<Operation> = vec![];

    while let Some(token) = parser.peek() {
        match token {
            Token::Op(op) => {
                while let Some(highest_op) = ops_stack.last() {
                    let top = highest_op;
                    let prec_op = parser.get_precedence(op);
                    let prec_top = parser.get_precedence(top);

                    if prec_op > prec_top
                        || (prec_op == prec_top && !parser.is_left_associative(op))
                    {
                        break;
                    }

                    if let Some(top_op) = ops_stack.pop() {
                        output.push(ArithmeticExpr::Aop(
                            top_op.to_string(),
                            Box::new(output.pop().unwrap()),
                            Box::new(output.pop().unwrap()),
                        ));
                    }
                }
                match parser.consume().unwrap() {
                    Token::Op(operation) => ops_stack.push(operation.clone()),
                    _ => {}
                }
            }
            Token::Num(num) => {
                parser.consume();
                output.push(ArithmeticExpr::Num(*num));
            }
            Token::Id(name) => {
                parser.consume();
                output.push(ArithmeticExpr::Var((name.clone()).parse().unwrap()));
            }
            _ => {}
        }
    }

    while let Some(op) = ops_stack.pop() {
        let highest_op = op;

        output.push(ArithmeticExpr::Aop(
            highest_op.to_string(),
            Box::new(output.pop().unwrap()),
            Box::new(output.pop().unwrap()),
        ));
    }
    if let Some(expr) = output.pop() {
        (Ok(expr), parser)
    } else {
        (Err(ParseError::UnexpectedToken(parser.peek().unwrap().clone())), parser)
    }
}

fn parse_call(mut parser: Parser) -> (ParseResult<ArithmeticExpr>, Parser) {
    match parser.peek() {
        Some(token) => {
            match token {
                Token::FnId(fn_name) => {
                    if parser.valid_function_name(fn_name.clone()) {
                        match parser.consume() {
                            Some(lbracket) => {
                                match lbracket {
                                    Token::LBracket => {
                                        // matches each function call with appropriate parameters and check validity
                                        let (result_params, mut parser_param) =
                                            parse_params(parser, fn_name);
                                        match result_params {
                                            Ok(params) => match parser_param.consume() {
                                                Some(rbracket) => match rbracket {
                                                    Token::RBracket => (
                                                        Ok(ArithmeticExpr::Call(
                                                            fn_name.clone(),
                                                            params,
                                                        )),
                                                        parser_param,
                                                    ),
                                                    _ => (
                                                        Err(ParseError::ExpectedToken(
                                                            Token::RBracket,
                                                        )),
                                                        parser_param,
                                                    ),
                                                },
                                                _ => (
                                                    Err(ParseError::ExpectedToken(Token::RBracket)),
                                                    parser_param,
                                                ),
                                            },
                                            Err(e) => (Err(e), parser_param),
                                        }
                                    }
                                    _ => (Err(ParseError::ExpectedToken(Token::LBracket)), parser),
                                }
                            }
                            _ => (Err(ParseError::ExpectedToken(Token::LBracket)), parser),
                        }
                    } else {
                        (
                            Err(ParseError::FunctionNotImplemented(fn_name.clone())),
                            parser,
                        )
                    }
                }
                _ => (
                    Err(ParseError::UnexpectedToken(parser.peek().unwrap().clone())),
                    parser,
                ),
            }
        }
        _ => (
            Err(ParseError::ExpectedToken(Token::FnId(
                "some_id".to_string(),
            ))),
            parser,
        ),
    }
}

fn parse_params(parser: Parser, fn_name: &String) -> (ParseResult<Vec<String>>, Parser) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenise;

    #[test]
    fn test_parse_program() {
        let mut parser = Parser::new(tokenise("skip;"));
        // println!("{:?}", parser.parse_program());
    }
}
