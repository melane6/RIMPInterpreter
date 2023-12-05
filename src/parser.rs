use std::collections::HashMap;
use std::env::Args;
use crate::lexer::{Keyword, Op, Token};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ParseError {
    UnexpectedToken(Token),
    SyntaxError(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RIMPProgram {
    Program(Stmts)
}
// While Grammar
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
    Function(String, Vec<String>, Block),
}
pub type Block = Vec<Stmt>;

// expression are either arithmetic or boolean
pub enum Expression {
    Aexp(ArithmeticExpr),
    Bexp(BooleanExpr),
}
pub type FnArgs = Vec<String>;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArithmeticExpr {
    Num(i64),
    Var(String),
    Aop(String, Box<ArithmeticExpr>, Box<ArithmeticExpr>),
    Call(String, Vec<ArithmeticExpr>),
    Uncall(String, Vec<ArithmeticExpr>),  // so tehniccally is allowed call foo1 (call foo2)
    //                                       but parse only args that are var or num
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BooleanExpr {
    True,
    False,
    Bop(String, ArithmeticExpr, ArithmeticExpr),
    Lop(String, Box<BooleanExpr>, Box<BooleanExpr>)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Parser {
    tokens: Vec<Token>,
    variables: HashMap<String, Expression>,
    functions: HashMap<String, Args>,
    index: usize,
}

type ParseResult<T> = Result<T, ParseError>;

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            variables: Default::default(),
            functions: Default::default(),
            index: 0,
        }
    }

    fn parse(&mut self) -> ParseResult<RIMPProgram> {
        let program = self.parse_program()?;
        Ok(program)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    fn consume(&mut self) -> Option<&Token> {
        let token = self.peek();
        self.index += 1;
        token
    }

    fn get_precedence(&self, op: &Op) -> i32 {
        match op {
            Op::Plus => 1,
            Op::Minus => 1,
            Op::Multiply => 2,
            Op::Divide => 2,
            Op::Mod => 2,
            Op::Power => 3,
            _ => 0
        }
    }

    fn parse_program(&mut self) -> ParseResult<RIMPProgram> {
        // program -> stmts
        match self.parse_stmts() {
            Ok(stmts) => Ok(RIMPProgram::Program(stmts)),
            Err(e) => Err(e)
        }
    }

    fn parse_stmts(&mut self) -> ParseResult<Stmts>{
        // stmts -> stmt stmts | ε
        match self.parse_stmt() {
            Ok(stmt) => {
                match self.parse_stmts() {
                    Ok(stmts) => Ok(Stmts::Statements(stmt, Box::new(stmts))),
                    Err(e) => Err(e)
                }
            },
            Err(e) => Err(e)
        }
    }

    fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        // stmt -> skip | x := aexp | if bexp then block else block | while bexp do block | Fn
        match self.peek() {
            Some(Token::Keyword(Keyword::Skip)) => { // skip
                self.consume();
                Ok(Stmt::Skip)
            },
            Some(Token::Id(x)) => { // assignment
                self.consume();
                match self.consume() {
                    Some(Token::Op(Op::Assign)) => {
                        match self.parse_aexp() {
                            Ok(aexp) => Ok(Stmt::Assign(x.clone(), aexp)),
                            Err(e) => Err(e)
                        }
                    },
                    Some(Token::Op(Op::Unassign)) => {
                        // Need to check that x is assigned
                        if true {
                            return Err(ParseError::SyntaxError("Variable is not assigned".to_string()));
                        }
                        match self.parse_aexp() {
                            Ok(aexp) => Ok(Stmt::Unassign(x.clone(), aexp)),
                            Err(e) => Err(e)
                        }
                    },
                    _ => Err(ParseError::UnexpectedToken(self.peek().unwrap().clone()))
                }
            },
            Some(Token::Keyword(Keyword::If)) => {
                self.consume();
                match self.parse_bexp() {
                    Ok(bexp) => {
                        match self.consume() {
                            Some(Token::Keyword(Keyword::Then)) => {
                                match self.parse_block() {
                                    Ok(block1) => {
                                        match self.consume() {
                                            Some(Token::Keyword(Keyword::Else)) => {
                                                match self.parse_block() {
                                                    Ok(block2) => Ok(Stmt::If(bexp, block1, block2)),
                                                    Err(e) => Err(e)
                                                }
                                            },
                                            _ => Err(ParseError::UnexpectedToken(self.peek().unwrap().clone()))
                                        }
                                    },
                                    _ => Err(ParseError::UnexpectedToken(self.peek().unwrap().clone()))
                                }
                            },
                            _ => Err(ParseError::UnexpectedToken(self.peek().unwrap().clone()))
                        }
                    },
                    _ =>Err(ParseError::UnexpectedToken(self.peek().unwrap().clone()))
                }
            },
            Some(Token::Keyword(Keyword::While)) => {
                self.consume();
                match self.parse_bexp() {
                    Ok(bexp) => {
                        match self.consume() {
                            Some(Token::Keyword(Keyword::Do)) => {
                                match self.parse_block() {
                                    Ok(block) => Ok(Stmt::While(bexp, block)),
                                    Err(e) => Err(e)
                                }
                            },
                            _ => Err(ParseError::UnexpectedToken(self.peek().unwrap().clone()))
                        }
                    },
                    Err(e) => Err(e)
                }
            },
            _ => {}
        }
    }

    fn parse_assign(&mut self) -> ParseResult<Stmt> {
        unimplemented!("Need to implement")
    }

    fn parse_unassign(&mut self) -> ParseResult<Stmt> {
        unimplemented!("Need to implement")
    }

    fn parse_if(&mut self) -> ParseResult<Stmt> {
        unimplemented!("Need to implement")
    }

    fn parse_while(&mut self) -> ParseResult<Stmt> {
        unimplemented!("Need to implement")
    }

    fn parse_aexp(&mut self) -> ParseResult<ArithmeticExpr> {
         unimplemented!("Need to implement")
    }

    fn parse_call(&mut self) -> ParseResult<ArithmeticExpr> {
        unimplemented!("Need to implement")
        // Check that fnIdentifier has been defined
        // Check that the arguments are valid?
        // do i need types here? harder to modfiy later?
    }

    fn parse_uncall(&mut self) -> ParseResult<ArithmeticExpr> {
        unimplemented!("Need to implement")
    }

    fn parse_bexp(&mut self) -> ParseResult<BooleanExpr> {
        unimplemented!("Need to implement")
    }

    fn parse_block(&mut self) -> ParseResult<Block> {
        unimplemented!("Need to implement")
    }

    fn parse_fn(&mut self) -> ParseResult<Stmt> {
        unimplemented!("Need to implement")
    }

    fn parse_args(&mut self) -> ParseResult<Vec<String>> {
        unimplemented!("Need to implement")
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{tokenise};

    #[test]
    fn test_parse_program() {
        let mut parser = Parser::new(tokenise("skip;"));
        println!("{:?}", parser.parse_program());
    }
}