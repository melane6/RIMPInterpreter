use crate::lexer::{tokenise, Keyword, Operation, Token};
use crate::parser::ArithmeticExpr::Aop;
use crate::parser::ParseErrorMessage::Errors;
use crate::transformations::{while_loop_transform, function_to_loop};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;
use std::{fmt, fs};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParseErrorMessage {
    UnexpectedToken(Token),
    SyntaxError(String),
    FunctionNotImplemented(String),
    ExpectedToken(Token),
    ExpectedOperation,
    ExpectedBoolOperation,
    EOF,
    Errors(Box<ParseErrorMessage>, Box<ParseErrorMessage>),
    MalformedProgram, //  The program must contain at least one statement or a block (i.e. a sequence of statements) or missed semicolon
    MalformedFunctionDefinition,
    MalformedBlock,
    MalformedIfStatement,
    MalformedWhileStatement,
    MalformedAssignment,
    MalformedFunctionCall,
    MalformedBooleanExpression,
    MalformedArithmeticExpression,
    MalformedArgumentList,
    NotAValidStatement,
    InvalidReturnStatement,
}

// Format the error message
impl ParseErrorMessage {
    pub fn to_string(&self) -> String {
        match self {
            ParseErrorMessage::UnexpectedToken(token) => format!("Unexpected token: {}", token.to_string()),
            ParseErrorMessage::SyntaxError(s) => format!("Syntax error: {}", s),
            ParseErrorMessage::FunctionNotImplemented(s) => {
                format!("Function not implemented: {}", s)
            }
            ParseErrorMessage::ExpectedToken(token) => format!("Expected token: {}", token.to_string()),
            ParseErrorMessage::ExpectedOperation => "Expected an arithmetic operation".to_string(),
            ParseErrorMessage::ExpectedBoolOperation => "Expected a boolean operation".to_string(),
            ParseErrorMessage::EOF => "Unexpected end of file or missing a semi colon".to_string(),
            Errors(e1, e2) => {
                format!("Error 1: {}\nError 2: {}", e1.to_string(), e2.to_string())
            }
            ParseErrorMessage::MalformedProgram => "Malformed program".to_string(),
            ParseErrorMessage::MalformedFunctionDefinition => "Malformed function definition".to_string(),
            ParseErrorMessage::MalformedBlock => "Malformed block".to_string(),
            ParseErrorMessage::MalformedIfStatement => "Malformed if statement".to_string(),
            ParseErrorMessage::MalformedWhileStatement => "Malformed while statement".to_string(),
            ParseErrorMessage::MalformedAssignment => "Malformed assignment".to_string(),
            ParseErrorMessage::MalformedFunctionCall => "Malformed function call".to_string(),
            ParseErrorMessage::MalformedBooleanExpression => "Malformed boolean expression".to_string(),
            ParseErrorMessage::MalformedArithmeticExpression => "Malformed arithmetic expression".to_string(),
            ParseErrorMessage::MalformedArgumentList => "Malformed argument list".to_string(),
            ParseErrorMessage::NotAValidStatement => "Not a valid statement".to_string(),
            ParseErrorMessage::InvalidReturnStatement => "Invalid return statement".to_string(),
        }
    }
}

pub type ParseError = (ParseErrorMessage, usize);

pub type Block = Vec<Stmt>;

pub type FnArgs = Vec<String>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stmt {
    Skip,
    Assign(String, ArithmeticExpr),
    Unassign(String, ArithmeticExpr),
    If(BooleanExpr, Block, Block),
    While(BooleanExpr, Block, BooleanExpr),
    WhileR(BooleanExpr, Block, BooleanExpr),
    Function(String, FnArgs, Block), // function name, arguments, body
    Return(ArithmeticExpr),
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stmt::Skip => write!(f, "skip"),
            Stmt::Assign(id, value) => write!(f, "{} := {}", id, value.to_string()),
            Stmt::Unassign(id, value) => write!(f, "{} =: {}", id, value.to_string()),
            Stmt::If(boolean, then_block, else_block) => {
                let mut if_stmt = format!("if {} then\n", boolean.to_string());
                for stmt in then_block {
                    if_stmt.push_str(&format!("  {}\n", stmt.to_string()));
                }
                if_stmt.push_str("else\n");
                for stmt in else_block {
                    if_stmt.push_str(&format!("  {}\n", stmt.to_string()));
                }
                write!(f, "{}", if_stmt)
            }
            Stmt::While(boolean, block, _) => {
                let mut while_stmt = format!("while {} do\n", boolean.to_string());
                for stmt in block {
                    while_stmt.push_str(&format!("  {}\n", stmt.to_string()));
                }
                write!(f, "{}", while_stmt)
            }
            Stmt::WhileR(_, block, boolean_acc) => {
                let mut while_stmt = format!("while {} do\n", boolean_acc.clone()); //todo check
                for stmt in block {
                    while_stmt.push_str(&format!("  {}\n", stmt.to_string()));
                }
                write!(f, "{}", while_stmt)
            }
            Stmt::Function(fn_name, args, block) => {
                let mut function = format!("fn {}(", fn_name);
                for arg in args {
                    if args.last().unwrap() == arg {
                        function.push_str(&format!("{}) {{\n", arg));
                    } else {
                        function.push_str(&format!("{}, ", arg));
                    }
                }
                function.push_str(") {\n");
                for stmt in block {
                    function.push_str(&format!("  {}\n", stmt.to_string()));
                }
                function.push_str("}");
                write!(f, "{}", function)
            }
            Stmt::Return(value) => write!(f, "return {}", value.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArithmeticExpr {
    Num(i64),
    LocalID(String),
    GlobalID(String),
    Aop(String, Box<ArithmeticExpr>, Box<ArithmeticExpr>),
    Call(String, Vec<ArithmeticExpr>),
}

impl fmt::Display for ArithmeticExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArithmeticExpr::Num(n) => write!(f, "{}", n),
            ArithmeticExpr::LocalID(id) => write!(f, "{}", id),
            ArithmeticExpr::GlobalID(id) => write!(f, "{}", id),
            ArithmeticExpr::Aop(op, a1, a2) => {
                write!(f, "({} {} {})", a1.to_string(), op, a2.to_string())
            }
            ArithmeticExpr::Call(fn_name, args) => {
                let mut call = format!("{}(", fn_name);
                for arg in args {
                    if args.last().unwrap() == arg {
                        call.push_str(&format!("{})", arg.to_string()));
                    } else {
                        call.push_str(&format!("{}, ", arg.to_string()));
                    }
                }
                call.push_str(")");
                write!(f, "{}", call)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BooleanExpr {
    True,
    False,
    Bop(String, ArithmeticExpr, ArithmeticExpr),
    Lop(String, Box<BooleanExpr>, Box<BooleanExpr>),
    Not(Box<BooleanExpr>),
}

impl fmt::Display for BooleanExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BooleanExpr::True => write!(f, "true"),
            BooleanExpr::False => write!(f, "false"),
            BooleanExpr::Bop(op, a1, a2) => {
                write!(f, "({} {} {})", a1.to_string(), op, a2.to_string())
            }
            BooleanExpr::Lop(op, b1, b2) => {
                write!(f, "({} {} {})", b1.to_string(), op, b2.to_string())
            }
            BooleanExpr::Not(b) => {
                write!(f, "!({})", b.to_string())
            }
        }
    }
}

#[test]
fn display_code() {
    let code = Stmt::Assign("x".to_string(), ArithmeticExpr::Num(1));
    let code2 = Stmt::Assign(
        "y".to_string(),
        Aop(
            "+".to_string(),
            Box::new(ArithmeticExpr::LocalID("x".to_string())),
            Box::new(ArithmeticExpr::Num(1)),
        ),
    );
    let while_loop = Stmt::While(
        BooleanExpr::Bop(
            "==".to_string(),
            ArithmeticExpr::LocalID("x".to_string()),
            ArithmeticExpr::Num(1),
        ),
        vec![Stmt::Assign(
            "x".to_string(),
            Aop(
                "+".to_string(),
                Box::new(ArithmeticExpr::LocalID("x".to_string())),
                Box::new(ArithmeticExpr::Num(1)),
            ),
        )],
        BooleanExpr::Bop(
            "!=".to_string(),
            ArithmeticExpr::LocalID("acc".to_string()),
            ArithmeticExpr::Num(0),
        )
    );
    println!("{:?}", code.to_string());
    println!("{:?}", code2.to_string());
    println!("{:?}", while_loop.to_string());
}
type TokenList = Vec<Token>;

trait IsSeq<T> {
    fn into_seq(self) -> Vec<T>;
}

// implement is Seq for String
impl IsSeq<char> for String {
    fn into_seq(self) -> Vec<char> {
        self.chars().collect()
    }
}

// implement is Seq for Vec<T>
impl<T> IsSeq<T> for Vec<T> {
    fn into_seq(self) -> Vec<T> {
        self
    }
}

type ParserResult<T, I> = HashSet<(T, I)>;

pub(crate) type ParseResult<T, I> = Result<HashSet<(T, I)>, ParseError>;
pub(crate) type ParseFinalResult<T> = Result<HashSet<T>, ParseError>;

fn parse_all<I, T>(parsed: ParserResult<T, Vec<I>>) -> HashSet<T>
where
    I: Debug + Clone + PartialEq + Eq + Hash + 'static,
    T: Debug + Clone + PartialEq + Eq + Hash + 'static,
{
    let mut result = HashSet::new();
    for (head, tail) in parsed {
        if tail.is_empty() {
            result.insert(head.clone());
            println!("{:?}", head);
        }
    }
    result
}

fn alt<'a, I, T, Left, Right>(
    left_parser: Left,
    right_parser: Right,
) -> impl Fn(I) -> ParseResult<T, I>
where
    T: Debug + Clone + PartialEq + Eq + Hash + 'static,
    I: Debug + Clone + PartialEq + Eq + Hash + 'static,
    Left: Fn(I) -> ParseResult<T, I>,
    Right: Fn(I) -> ParseResult<T, I>,
{
    move |ts: I| {
        let mut result = HashSet::new();
        let left_result = left_parser(ts.clone());
        let right_result = right_parser(ts.clone());
        match (left_result, right_result) {
            (Ok(left), Ok(right)) => {
                result.extend(left);
                result.extend(right);
                Ok(result)
            }
            (Ok(left), Err(_)) => Ok(left),
            (Err(_), Ok(right)) => Ok(right),
            (Err(left_error), Err(right_error)) => {
                if left_error.1 > right_error.1 {
                    Err(left_error)
                } else {
                    Err(right_error)
                }
            }
        }
    }
}

fn seq<'a, I, T, S, Left, Right>(
    left_parser: Left,
    right_parser: Right,
) -> impl Fn(I) -> ParseResult<(T, S), I>
where
    I: Debug + Clone + PartialEq + Eq + Hash + 'static,
    T: Debug + Clone + PartialEq + Eq + Hash + 'static,
    S: Debug + Clone + PartialEq + Eq + Hash + 'static,
    Left: Fn(I) -> ParseResult<T, I>,
    Right: Fn(I) -> ParseResult<S, I>,
{
    move |ts: I| {
        let mut result = HashSet::new();
        let left_result = left_parser(ts.clone());
        match left_result {
            Ok(left) => {
                let mut no_match = false;
                let mut error = (ParseErrorMessage::SyntaxError("".to_string()), 0);
                for (head, tail) in left {
                    let right_result = right_parser(tail);
                    match right_result {
                        Ok(right) => {
                            for (head2, tail2) in right {
                                result.insert(((head.clone(), head2.clone()), tail2.clone()));
                            }
                        }
                        Err(e) => {
                            no_match = true;
                            error = e;
                        }
                    }
                }
                if result.is_empty() && no_match {
                    Err(error)
                } else {
                    Ok(result)
                }
            }
            Err(e) => {
                Err(e) 
            },
        }
    }
}

#[test]
fn seq_test() {
    let tokens = vec![
        Token::Op(Operation::Assign),
        Token::Id("x".to_string()),
        Token::Op(Operation::Assign),
        Token::Num(1),
        Token::Num(1),
    ];
    let result = alt(
        map(
            seq(stmt(), seq(op_parser(), number_parser())),
            |(a, (op, b))| op,
        ),
        op_parser(),
    )(tokens);
    println!("{:?}", result);
}

fn map<'a, I, T, S, P, F>(parser: P, f: F) -> impl Fn(I) -> ParseResult<S, I>
where
    I: Debug + Clone + PartialEq + Eq + Hash + 'static,
    T: Debug + Clone + PartialEq + Eq + Hash + 'static,
    S: Debug + Clone + PartialEq + Eq + Hash + 'static,
    P: Fn(I) -> ParseResult<T, I>,
    F: Fn(T) -> S + 'a,
{
    move |ts: I| {
        let mut result = HashSet::new();
        let parser_result = parser(ts.clone());
        match parser_result {
            Ok(parsed) => {
                for (head, tail) in parsed {
                    result.insert(((f)(head), tail));
                }
                Ok(result)
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Parser {
    tokens: TokenList,
    variables: HashSet<String>,              // variable id, value?
    functions: HashMap<String, Vec<String>>, // function id, arguments and variable in that scope
    index: i64,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            tokens: vec![],
            variables: Default::default(),
            functions: Default::default(),
            index: 0,
        }
    }

    fn add_variable(&mut self, variable: String) {
        self.variables.insert(variable);
    }

    fn add_function(&mut self, function: String, args: Vec<String>) {
        self.functions.insert(function, args);
    }
}

fn id_token_parser() -> impl Fn(Vec<Token>) -> ParseResult<String, Vec<Token>> {
    move |ts: Vec<Token>| {
        let mut result = HashSet::new();
        if ts.clone().first().is_some() {
            match ts.first().clone().unwrap() {
                Token::Id(x) => {
                    result.insert((x.clone(), ts[1..].to_vec()));
                    Ok(result)
                }
                _ => Err((
                    ParseErrorMessage::ExpectedToken(Token::Id(
                        "Some variable id".parse().unwrap(),
                    )),
                    ts.len(),
                )),
            }
        } else {
            Err((ParseErrorMessage::EOF, ts.len()))
        }
    }
}

#[test]
fn id_token_parser_test() {
    let tokens = vec![Token::Op(":=".to_string().parse().unwrap()), Token::Num(1)];
    let result = id_token_parser()(tokens);
    println!("{:?}", result);
}

fn global_id_parser() -> impl Fn(Vec<Token>) -> ParseResult<String, Vec<Token>> {
    move |ts: Vec<Token>| {
        let mut result = HashSet::new();
        if ts.clone().first().is_some() {
            match ts.first().clone().unwrap() {
                Token::GlobalId(x) => {
                    result.insert((x.clone(), ts[1..].to_vec()));
                    Ok(result)
                }
                _ => Err((
                    ParseErrorMessage::ExpectedToken(Token::GlobalId(
                        "Some global variable".parse().unwrap(),
                    )),
                    ts.len(),
                )),
            }
        } else {
            Err((ParseErrorMessage::EOF, ts.len()))
        }
    }
}

fn function_id_token_parser() -> impl Fn(Vec<Token>) -> ParseResult<String, Vec<Token>> {
    move |ts: Vec<Token>| {
        let mut result = HashSet::new();
        if ts.clone().first().is_some() {
            match ts.first().clone().unwrap() {
                Token::FnId(x) => {
                    result.insert((x.clone(), ts[1..].to_vec()));
                    Ok(result)
                }
                _ => Err((
                    ParseErrorMessage::ExpectedToken(Token::Id(
                        "Some function id".parse().unwrap(),
                    )),
                    ts.len(),
                )),
            }
        } else {
            Err((ParseErrorMessage::ExpectedToken(
                Token::Id("Some function id".parse().unwrap()),
            ), ts.len()))
        }
    }
}

#[test]
fn fn_id_token_parser_test() {
    let tokens = vec![Token::Op(":=".to_string().parse().unwrap()), Token::Num(1)];
    let result = id_token_parser()(tokens);
    println!("{:?}", result);
}

fn number_parser() -> impl Fn(Vec<Token>) -> ParseResult<i64, Vec<Token>> {
    move |ts: Vec<Token>| {
        let mut result = HashSet::new();
        if ts.clone().first().is_some() {
            match ts.first().clone().unwrap() {
                Token::Num(x) => {
                    result.insert((x.clone(), ts[1..].to_vec()));
                    Ok(result)
                }
                _ => Err((ParseErrorMessage::ExpectedToken(Token::Num(0)), ts.len())),
            }
        } else {
            Err((ParseErrorMessage::EOF, ts.len()))
        }
    }
}

#[test]
fn test_num_token_parser() {
    let tokens = vec![Token::Num(1)];
    let result = number_parser()(tokens);
    println!("{:?}", result);
}

fn parenthesis(s: String) -> impl Fn(Vec<Token>) -> ParseResult<Token, Vec<Token>> {
    move |ts: Vec<Token>| {
        let mut result = HashSet::new();
        match s.as_str() {
            "(" => {
                if ts.first().is_some() && *ts.first().unwrap() == Token::LBracket {
                    result.insert((Token::LBracket, ts[1..].to_vec()));
                    Ok(result)
                } else {
                    Err((ParseErrorMessage::ExpectedToken(Token::LBracket), ts.len()))
                }
            }
            ")" => {
                if ts.first().is_some() && *ts.first().unwrap() == Token::RBracket {
                    result.insert((Token::RBracket, ts[1..].to_vec()));
                    Ok(result)
                } else {
                    Err((ParseErrorMessage::ExpectedToken(Token::RBracket), ts.len()))
                }
            }
            "{" => {
                if ts.first().is_some() && *ts.first().unwrap() == Token::LParent {
                    result.insert((Token::LParent, ts[1..].to_vec()));
                    Ok(result)
                } else {
                    Err((ParseErrorMessage::ExpectedToken(Token::LParent), ts.len()))
                }
            }
            "}" => {
                if ts.first().is_some() && *ts.first().unwrap() == Token::RParent {
                    result.insert((Token::RParent, ts[1..].to_vec()));
                    Ok(result)
                } else {
                    Err((ParseErrorMessage::ExpectedToken(Token::RParent), ts.len()))
                }
            }
            _ => panic!("Invalid parenthesis passed in to match token"),
        }
    }
}

#[test]
fn parenthesis_test() {
    let tokens = vec![Token::LBracket, Token::Num(1), Token::RBracket];
    let result = parenthesis("(".to_string())(tokens);
    println!("{:?}", result);
    let tokens = vec![Token::RBracket, Token::Num(1), Token::RParent];
    let result = parenthesis(")".to_string())(tokens);
    println!("{:?}", result);
    let tokens = vec![Token::LParent, Token::Num(1), Token::RParent];
    let result = parenthesis("{".to_string())(tokens);
    println!("{:?}", result);
    let tokens = vec![Token::RParent, Token::Num(1), Token::RParent];
    let result = parenthesis("}".to_string())(tokens);
    println!("{:?}", result);
}

fn comma_parser() -> impl Fn(Vec<Token>) -> ParseResult<String, Vec<Token>> {
    move |ts| {
        let mut result = HashSet::new();
        if ts.first().is_some() {
            if *ts.first().unwrap() == Token::Comma {
                result.insert((",".to_string(), ts[1..].to_vec()));
                Ok(result)
            } else {
                Err((ParseErrorMessage::ExpectedToken(Token::Comma), ts.len()))
            }
        } else {
            Err((ParseErrorMessage::EOF, ts.len()))
        }
    }
}

fn semi_colon_parser() -> impl Fn(Vec<Token>) -> ParseResult<String, Vec<Token>> {
    move |ts| {
        let mut result = HashSet::new();
        if ts.first().is_some() {
            if *ts.first().unwrap() == Token::Semi {
                result.insert((";".to_string(), ts[1..].to_vec()));
                Ok(result)
            } else {
                Err((ParseErrorMessage::ExpectedToken(Token::Semi), ts.len()))
            }
        } else {
            Err((ParseErrorMessage::EOF, ts.len()))
        }
    }
}

fn op_parser() -> impl Fn(Vec<Token>) -> ParseResult<Operation, Vec<Token>> {
    move |ts: Vec<Token>| {
        let mut result = HashSet::new();
        if ts.first().is_some() {
            match ts.first().unwrap() {
                Token::Op(s) => match s {
                    Operation::Plus | Operation::Minus => {
                        result.insert((s.clone(), ts[1..].to_vec()));
                        Ok(result)
                    }
                    _ => Err((ParseErrorMessage::ExpectedOperation, ts.len())),
                },
                _ => Err((ParseErrorMessage::ExpectedOperation, ts.len())),
            }
        } else {
            Err((ParseErrorMessage::EOF, ts.len()))
        }
    }
}

fn prior_op_parser() -> impl Fn(Vec<Token>) -> ParseResult<Operation, Vec<Token>> {
    move |ts: Vec<Token>| {
        let mut result = HashSet::new();
        if ts.first().is_some() {
            match ts.first().unwrap() {
                Token::Op(s) => match s {
                    Operation::Multiply | Operation::Divide | Operation::Mod => {
                        result.insert((s.clone(), ts[1..].to_vec()));
                        Ok(result)
                    }
                    _ => Err((ParseErrorMessage::ExpectedOperation, ts.len())),
                },
                _ => Err((ParseErrorMessage::ExpectedOperation, ts.len())),
            }
        } else {
            Err((ParseErrorMessage::EOF, ts.len()))
        }
    }
}

#[test]
fn op_test() {
    let tokens = vec![Token::Op(Operation::Plus), Token::Num(1)];
    let result = op_parser()(tokens);
    println!("{:?}", result);
}

fn bop_parser() -> impl Fn(Vec<Token>) -> ParseResult<Operation, Vec<Token>> {
    move |ts: Vec<Token>| {
        let mut result = HashSet::new();
        if ts.first().is_some() {
            match ts.first().unwrap() {
                Token::Op(s) => match s {
                    Operation::Equal
                    | Operation::LessThan
                    | Operation::GreaterThan
                    | Operation::LessThanEqual
                    | Operation::GreaterThanEqual
                    | Operation::NotEqual => {
                        result.insert((s.clone(), ts[1..].to_vec()));
                        Ok(result)
                    }
                    _ => Err((ParseErrorMessage::ExpectedBoolOperation, ts.len())),
                },
                _ => Err((ParseErrorMessage::ExpectedBoolOperation, ts.len())),
            }
        } else {
            Err((ParseErrorMessage::EOF, ts.len()))
        }
    }
}

#[test]
fn bop_test() {
    let tokens = vec![Token::Op(Operation::NotEqual), Token::Num(1)];
    let result = bop_parser()(tokens);
    println!("{:?}", result);
}

fn not_parser() -> impl Fn(Vec<Token>) -> ParseResult<Operation, Vec<Token>> {
    move |ts: Vec<Token>| {
        let mut result = HashSet::new();
        if ts.first().is_some() {
            match ts.first().unwrap() {
                Token::Op(s) => match s {
                    Operation::Not => {
                        result.insert((s.clone(), ts[1..].to_vec()));
                        Ok(result)
                    }
                    _ => Err((ParseErrorMessage::ExpectedBoolOperation, ts.len())),
                },
                _ => Err((ParseErrorMessage::ExpectedBoolOperation, ts.len())),
            }
        } else {
            Err((ParseErrorMessage::EOF, ts.len()))
        }
    }
}

fn lop_token_parser() -> impl Fn(Vec<Token>) -> ParseResult<Operation, Vec<Token>> {
    move |ts: Vec<Token>| {
        let mut result = HashSet::new();
        if ts.first().is_some() {
            match ts.first().unwrap() {
                Token::Op(s) => match s {
                    Operation::And | Operation::Or => {
                        result.insert((s.clone(), ts[1..].to_vec()));
                        Ok(result)
                    }
                    _ => Err((ParseErrorMessage::ExpectedBoolOperation, ts.len())),
                },
                _ => Err((ParseErrorMessage::ExpectedBoolOperation, ts.len())),
            }
        } else {
            Err((ParseErrorMessage::EOF, ts.len()))
        }
    }
}

#[test]
fn lop_test() {
    let tokens = vec![Token::Op(Operation::Or), Token::Num(1)];
    let result = lop_token_parser()(tokens);
    println!("{:?}", result);
}

fn symbol_parser(s: String) -> impl Fn(Vec<Token>) -> ParseResult<String, Vec<Token>> {
    move |ts: Vec<Token>| {
        let mut result = HashSet::new();
        if ts.first().is_some() {
            match ts.first().unwrap() {
                Token::Op(symbol) => {
                    if s.clone() == symbol.to_string() {
                        result.insert((s.clone(), ts[1..].to_vec()));
                        Ok(result)
                    } else {
                        Err((
                            ParseErrorMessage::ExpectedToken(Token::String(
                                "Expected some symbol".to_string(),
                            )),
                            ts.len(),
                        ))
                    }
                }
                _ => Err((
                    ParseErrorMessage::ExpectedToken(Token::String(
                        "Expected some symbol".to_string(),
                    )),
                    ts.len(),
                )),
            }
        } else {
            Err((ParseErrorMessage::EOF, ts.len()))
        }
    }
}

#[test]
fn symbol_test() {
    let tokens = vec![Token::Op(Operation::Minus), Token::Num(1)];
    let result = symbol_parser("-".to_string())(tokens);
    println!("{:?}", result);
}

fn keyword_parser(s: String) -> impl Fn(Vec<Token>) -> ParseResult<String, Vec<Token>> {
    move |ts: Vec<Token>| {
        let mut result = HashSet::new();
        if ts.first().is_some() {
            match ts.first().unwrap() {
                Token::Keyword(keyword) => {
                    if s.clone() == keyword.to_string() {
                        result.insert((s.clone(), ts[1..].to_vec()));
                        Ok(result)
                    } else {
                        Err((
                            ParseErrorMessage::ExpectedToken(Token::Keyword(
                                Keyword::from_str(s.as_str()).unwrap(),
                            )),
                            ts.len(),
                        ))
                    }
                }
                _ => Err((
                    ParseErrorMessage::ExpectedToken(Token::Keyword(
                        Keyword::from_str(s.as_str()).unwrap(),
                    )),
                    ts.len(),
                )),
            }
        } else {
            Err((ParseErrorMessage::EOF, ts.len()))
        }
    }
}

#[test]
fn keyword_test() {
    let tokens = vec![Token::Keyword(Keyword::Skip), Token::Num(1)];
    let result = keyword_parser("skip".to_string())(tokens);
    println!("{:?}", result);
}

fn skip_parser() -> impl Fn(Vec<Token>) -> ParseResult<Stmt, Vec<Token>> {
    move |ts| {
        let mut result = HashSet::new();
        if ts.first().is_some() {
            match ts.first().unwrap() {
                Token::Keyword(s) => match s {
                    Keyword::Skip => {
                        result.insert((Stmt::Skip, ts[1..].to_vec()));
                        Ok(result)
                    }
                    _ => Err((
                        ParseErrorMessage::ExpectedToken(Token::Keyword(Keyword::Skip)),
                        ts.len(),
                    )),
                },
                _ => Err((
                    ParseErrorMessage::ExpectedToken(Token::Keyword(Keyword::Skip)),
                    ts.len(),
                )),
            }
        } else {
            Err((ParseErrorMessage::EOF, ts.len()))
        }
    }
}

#[test]
fn skip_test() {
    let tokens = vec![Token::Keyword(Keyword::Skip), Token::Num(1)];
    let result = skip_parser()(tokens);
    println!("{:?}", result);
}

fn arithmetic_expr() -> impl Fn(Vec<Token>) -> ParseResult<ArithmeticExpr, Vec<Token>> {
    move |ts| match alt(
        map(
            seq(term(), seq(op_parser(), arithmetic_expr())),
            |(fa, (op, a))| ArithmeticExpr::Aop(op.to_string(), Box::new(fa), Box::new(a)),
        ),
        term(),
    )(ts.clone())
    {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err((
            Errors(
                Box::from(e.0),
                Box::from(ParseErrorMessage::MalformedArithmeticExpression),
            ),
            ts.len(),
        )),
    }
}

#[test]
fn arithmetic_expr_test() {
    let tokens = vec![
        Token::Id("x".to_string()),
        Token::LBracket,
        Token::Num(1),
        Token::RBracket,
        Token::Op(Operation::Plus),
        Token::Num(1),
    ];
    let result = arithmetic_expr()(tokens);
    println!("{:?}", result);
}

fn term() -> impl Fn(Vec<Token>) -> ParseResult<ArithmeticExpr, Vec<Token>> {
    move |ts| {
        alt(
            map(
                seq(factor(), seq(prior_op_parser(), term())),
                |(fa, (op, a))| ArithmeticExpr::Aop(op.to_string(), Box::new(fa), Box::new(a)),
            ),
            factor(),
        )(ts)
    }
}

#[test]
fn term_test() {
    let tokens = vec![Token::Num(1), Token::Op(Operation::Plus), Token::Num(1)];
    let result = term()(tokens);
    println!("{:?}", result);
}

fn call() -> impl Fn(Vec<Token>) -> ParseResult<ArithmeticExpr, Vec<Token>> {
    move |ts| match map(
        seq(
            id_token_parser(),
            map(
                seq(
                    parenthesis("(".to_string()),
                    seq(params(), parenthesis(")".to_string())),
                ),
                |(_, (params, _))| params,
            ),
        ),
        |(id, params)| ArithmeticExpr::Call(id, params),
    )(ts)
    {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err((
            Errors(
                Box::from(e.0),
                Box::from(ParseErrorMessage::MalformedFunctionCall),
            ),
            e.1,
        )),
    }
}

#[test]
fn call_test() {
    let tokens = vec![
        Token::Id("x".to_string()),
        Token::LBracket,
        Token::Num(1),
        Token::Comma,
        Token::Num(1),
        Token::RBracket,
    ];
    let result = call()(tokens);
    println!("{:?}", result);
}

fn params() -> impl Fn(Vec<Token>) -> ParseResult<Vec<ArithmeticExpr>, Vec<Token>> {
    move |ts| match alt(
        map(
            seq(arithmetic_expr(), seq(comma_parser(), params())),
            |(param, (_, mut params))| {
                params.insert(0, param);
                params
            },
        ),
        map(arithmetic_expr(), |param| vec![param]),
    )(ts.clone())
    {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err((
            Errors(
                Box::from(e.0),
                Box::from(ParseErrorMessage::MalformedArgumentList),
            ),
            ts.len(),
        )),
    }
}

#[test]
fn params_test() {
    let tokens = vec![
        Token::Id("x".to_string()),
        Token::Comma,
        Token::Id("y".to_string()),
    ];
    let result = params()(tokens);
    println!("{:?}", result);
}

fn factor() -> impl Fn(Vec<Token>) -> ParseResult<ArithmeticExpr, Vec<Token>> {
    move |ts| {
        alt(
            call(),
            alt(
                map(number_parser(), |num| ArithmeticExpr::Num(num)),
                alt(
                    map(id_token_parser(), |id| ArithmeticExpr::LocalID(id)),
                    map(global_id_parser(), |id| ArithmeticExpr::GlobalID(id)),
                ),
            ),
        )(ts)
    }
}

fn uni_bool_expr() -> impl Fn(Vec<Token>) -> ParseResult<BooleanExpr, Vec<Token>> {
    move |ts| {
        alt(
            map(
                seq(not_parser(), boolean_expr()),
                |(_, b)| BooleanExpr::Not(Box::new(b)),
            ),
            map(
                seq(boolean_expr(), seq(lop_token_parser(), boolean_expr())),
                |(b1, (op, b2))| BooleanExpr::Lop(op.to_string(), Box::new(b1), Box::new(b2)),
            ),
        )(ts)
    }
}

fn boolean_expr() -> impl Fn(Vec<Token>) -> ParseResult<BooleanExpr, Vec<Token>> {
    move |ts| {
        alt(
            map(
                seq(arithmetic_expr(), seq(bop_parser(), arithmetic_expr())),
                |(a1, (op, a2))| BooleanExpr::Bop(op.to_string(), a1, a2),
            ),
            map(
                seq(
                    parenthesis("(".to_string()),
                    seq(boolean_expr(), parenthesis(")".to_string())),
                ),
                |(_, (b, _))| b,
            ),
        )(ts)
    }
}

#[test]
fn boolean_expr_test() {
    let tokens = vec![Token::Num(1), Token::Op(Operation::NotEqual), Token::Num(1)];
    let result = boolean_expr()(tokens);
    println!("{:?}", result);
}

fn element() -> impl Fn(Vec<Token>) -> ParseResult<String, Vec<Token>> {
    move |ts| {
        alt(
            map(
                seq(
                    parenthesis("(".to_string()),
                    seq(id_token_parser(), parenthesis(")".to_string())),
                ),
                |(_, (s, _))| s,
            ),
            id_token_parser(),
        )(ts)
    }
}


fn if_parser() -> impl Fn(Vec<Token>) -> ParseResult<Stmt, Vec<Token>> {
    move |ts| match map(
        seq(
            keyword_parser("if".to_string()),
            seq(
                alt(boolean_expr(), uni_bool_expr()),
                seq(
                    keyword_parser("then".to_string()),
                    seq(block(), seq(keyword_parser("else".to_string()), block())),
                ),
            ),
        ),
        |(_, (b, (_, (a1, (_, a2)))))| Stmt::If(b, a1, a2),
    )(ts.clone())
    {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err((
            Errors(
                Box::from(e.0),
                Box::from(ParseErrorMessage::MalformedIfStatement),
            ),
            e.1,
        )
        ),
    }
}

#[test]
fn if_parser_test() {
    let tokens = vec![
        Token::Keyword(Keyword::If),
        Token::Num(1),
        Token::Op(Operation::NotEqual),
        Token::Keyword(Keyword::Then),
        Token::Keyword(Keyword::Skip),
        Token::Keyword(Keyword::Else),
        Token::Keyword(Keyword::Skip),
    ];
    let result = if_parser()(tokens);
    println!("{:?}", result);
}

fn while_parser() -> impl Fn(Vec<Token>) -> ParseResult<Block, Vec<Token>> {
    move |ts| match map(
        seq(
            keyword_parser("while".to_string()),
            seq(
                alt(boolean_expr(), uni_bool_expr()),
                seq(
                    keyword_parser("do".to_string()),
                    seq(
                        parenthesis("{".to_string()),
                        seq(block(), parenthesis("}".to_string())),
                    ),
                ),
            ),
        ),
        |(_, (b, (_, (_, (block, _)))))| while_loop_transform(Stmt::While(b, block, BooleanExpr::True)),
    )(ts.clone())
    {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err((
            Errors(
                Box::from(e.0),
                Box::from(ParseErrorMessage::MalformedWhileStatement),
            ),
            ts.len(),
        )),
    }
}

#[test]
fn while_parser_test() {
    let tokens = vec![
        Token::Keyword(Keyword::While),
        Token::Num(1),
        Token::Op(Operation::NotEqual),
        Token::Num(1),
        Token::Keyword(Keyword::Do),
        Token::LParent,
        Token::Keyword(Keyword::Skip),
        Token::Semi,
        Token::Keyword(Keyword::Skip),
        Token::RParent,
    ];
    let result = while_parser()(tokens);
    println!("{:?}", result);
}

fn block() -> impl Fn(Vec<Token>) -> ParseResult<Block, Vec<Token>> {
    move |ts| match alt(
        map(
            seq(
                parenthesis("{".to_string()),
                seq(
                    stmts(),
                    parenthesis("}".to_string()),
                ),
            ),
            |(_, (s, _))| s,
        ),
        map(
            stmt(),
            |s| vec![s],
        ),
    )(ts)
    {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err((
            Errors(Box::from(e.0), Box::from(ParseErrorMessage::MalformedBlock)),
            e.1,
        )),
    }
}

#[test]
fn block_test() {
    let tokens = vec![
        Token::Keyword(Keyword::Skip),
        Token::Semi,
        Token::Keyword(Keyword::Skip),
    ];
    let result = block()(tokens);
    println!("{:?}", result);
}

fn assignment() -> impl Fn(Vec<Token>) -> ParseResult<Stmt, Vec<Token>> {
    move |ts| match map(
        seq(
            alt(id_token_parser(), global_id_parser()),
            seq(symbol_parser(":=".to_string()), arithmetic_expr()),
        ),
        |(id, (_, a))| Stmt::Assign(id, a),
    )(ts)
    {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err((
            Errors(
                Box::from(e.0),
                Box::from(ParseErrorMessage::MalformedAssignment),
            ),
            e.1,
        )),
    }
}

fn return_statement() -> impl Fn(Vec<Token>) -> ParseResult<Stmt, Vec<Token>> {
    move |ts| match map(
        seq(keyword_parser("return".to_string()), arithmetic_expr()),
        |(_, a)| Stmt::Return(a),
    )(ts)
    {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err((
            Errors(
                Box::from(e.0),
                Box::from(ParseErrorMessage::InvalidReturnStatement),
            ),
            e.1,
        )),
    }
}

#[test]
fn return_statement_test() {
    let tokens = vec![Token::Keyword(Keyword::Return), Token::Num(1)];
    let result = return_statement()(tokens);
    println!("{:?}", result);
}

fn stmt() -> impl Fn(Vec<Token>) -> ParseResult<Stmt, Vec<Token>> {
    move |ts| match alt(
        function_parser(),
        alt(
            if_parser(),
            alt(assignment(), alt(return_statement(), skip_parser())),
        ),
    )(ts)
    {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err((
            Errors(
                Box::from(e.0),
                Box::from(ParseErrorMessage::NotAValidStatement),
            ),
            e.1,
        )),
    }
}

#[test]
fn stmt_test() {
    let tokens = vec![
        Token::GlobalId("X".to_string()),
        Token::Op(Operation::Assign),
        Token::Id("x".to_string()),
        Token::LBracket,
        Token::Num(1),
        Token::RBracket,
    ];
    let result = stmt()(tokens);
    println!("{:?}", result);
}

fn function_id() -> impl Fn(Vec<Token>) -> ParseResult<String, Vec<Token>> {
    move |vec| {
        let mut result = HashSet::new();
        if vec.first().is_some() {
            match vec.first().unwrap() {
                Token::Keyword(keyword) => match keyword {
                    Keyword::Def => match id_token_parser()(vec[1..].to_vec()) {
                        Ok(parsed) => {
                            result.insert((
                                parsed.iter().next().unwrap().0.clone(),
                                vec[2..].to_vec(),
                            ));
                            Ok(result)
                        }
                        Err(e) => Err(e),
                    },
                    _ => Err((
                        ParseErrorMessage::ExpectedToken(Token::Keyword(Keyword::Def)),
                        vec.len(),
                    )),
                },
                _ => Err((
                    ParseErrorMessage::ExpectedToken(Token::Keyword(Keyword::Def)),
                    vec.len(),
                )),
            }
        } else {
            Err((ParseErrorMessage::EOF, vec.len()))
        }
    }
}

#[test]
fn function_id_test() {
    let tokens = vec![
        Token::Keyword(Keyword::Def),
        Token::FnId("main".to_string()),
    ];
    let result = function_id()(tokens);
    println!("{:?}", result);
}

fn args() -> impl Fn(Vec<Token>) -> ParseResult<Vec<String>, Vec<Token>> {
    move |ts| match alt(
        map(
            seq(id_token_parser(), seq(comma_parser(), args())),
            |(id, (_, mut args))| {
                args.insert(0, id);
                args
            },
        ),
        map(id_token_parser(), |id| vec![id]),
    )(ts)
    {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err((
            Errors(
                Box::from(e.0),
                Box::from(ParseErrorMessage::MalformedArgumentList),
            ),
            e.1,
        )),
    }
}

#[test]
fn args_test() {
    let tokens = vec![
        Token::Id("x".to_string()),
        Token::Comma,
        Token::Id("y".to_string()),
    ];
    let result = args()(tokens);
    println!("{:?}", result);
}

// Function parser is a parser that parses a function
// definition in the form of:
// fn <function_id>(<params>) { <block> }
fn function_parser() -> impl Fn(Vec<Token>) -> ParseResult<Stmt, Vec<Token>> {
    move |ts| match map(
        seq(
            function_id(),
            seq(
                map(
                    seq(
                        parenthesis("(".to_string()),
                        seq(args(), parenthesis(")".to_string())),
                    ),
                    |(_, (arguments, _))| arguments,
                ),
                seq(
                    parenthesis("{".to_string()),
                    seq(stmts(), parenthesis("}".to_string())),
                ),
            ),
        ),
        |(id, (args, (_, (block, _))))| function_to_loop(id, block, args),
    )(ts)
    {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err((
            Errors(
                Box::from(e.0),
                Box::from(ParseErrorMessage::MalformedFunctionDefinition),
            ),
            e.1,
        )),
    }
}

#[test]
fn function_parser_test() {
    let tokens = vec![
        Token::Keyword(Keyword::Def),
        Token::Id("main".to_string()),
        Token::LBracket,
        Token::Id("x".to_string()),
        Token::Comma,
        Token::Id("y".to_string()),
        Token::RBracket,
        Token::LParent,
        Token::Id("x".to_string()),
        Token::Op(Operation::Assign),
        Token::Num(1),
        Token::Semi,
        Token::Keyword(Keyword::Skip),
        Token::Semi,
        Token::Keyword(Keyword::Return),
        Token::Num(1),
        Token::RParent,
    ];
    let result = function_parser()(tokens.clone());
    let len = tokens.len();
    println!("{:?}", tokens.clone());
    println!("{:?}", result);
    println!("{:?}", tokens[len - 20]);
}

fn stmts() -> impl Fn(Vec<Token>) -> ParseResult<Block, Vec<Token>> {
    move |ts| match alt(
        map(
            seq(stmt(), seq(semi_colon_parser(), stmts())),
            |(stmt, (_, mut stmts))| {
                stmts.insert(0, stmt);
                stmts
            },
        ),
        alt(
            map(
                seq(while_parser(), seq(semi_colon_parser(), stmts())),
                |(mut block, (_, block_1))| {
                    block.extend(block_1);
                    block
                },
            ),
            alt(map(stmt(), |stmt| vec![stmt]), while_parser()),
        ),
    )(ts.clone())
    {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err((
            Errors(
                Box::from(e.0),
                Box::from(ParseErrorMessage::MalformedProgram),
            ),
            e.1,
        )),
    }
}

#[test]
fn stmts_test() {
    let tokens = vec![
        Token::Keyword(Keyword::Def),
        Token::Id("main".to_string()),
        Token::LBracket,
        Token::Id("x".to_string()),
        Token::Comma,
        Token::Id("y".to_string()),
        Token::RBracket,
        Token::LParent,
        Token::Id("x".to_string()),
        Token::Op(Operation::Assign),
        Token::Num(1),
        Token::Semi,
        Token::Keyword(Keyword::While),
        Token::Num(1),
        Token::Op(Operation::NotEqual),
        Token::Num(1),
        Token::Keyword(Keyword::Do),
        Token::LParent,
        Token::Keyword(Keyword::Skip),
        Token::RParent,
        Token::RParent,
    ];
    let result = function_parser()(tokens);
    println!("{:?}", result);
}

#[test]
fn parse_program_test() {
    let tokens = tokenise("n := 4  Fact := 1");
    let tokens = match tokens {
        Ok(tokens) => tokens,
        Err(e) => {
            println!("Failed to tokenise program. {}", e);
            return;
        }
    };
    println!("{:?}", tokens);
    let result = stmts()(tokens);
    println!("{:?}", result);
}

pub(crate) fn parse_rimp(tokens: Vec<Token>) -> Result<HashSet<Block>, ParseError> {
    let parsed = stmts()(tokens.clone());
    match parsed {
        Ok(parsed) => {
            let mut result = HashSet::new();
            let mut tokens_not_processed = 0;
            for (head, tail) in parsed.clone() {
                if tail.is_empty() {
                    result.insert(head.clone());
                } else {
                    println!("tal{:?}", tail);
                    tokens_not_processed = tail.len();
                }
            }
            if result.is_empty() {
                Err((ParseErrorMessage::ExpectedToken(Token::Semi), tokens_not_processed))
            } else {
                Ok(result)
            }
        }
        Err(e) => {
            Err(e)
        },
    }
}

fn parse_program(tokens: Vec<Token>) -> Result<HashSet<Block>, ParseError> {
    let parsed = stmts()(tokens);
    match parsed {
        Ok(parsed) => {
            let mut result = HashSet::new();
            for (head, tail) in parsed {
                if tail.is_empty() {
                    result.insert(head.clone());
                } else {
                    if tail.first().is_some() {
                        if *tail.first().unwrap() == Token::Semi {
                            let new_tail = tail[1..].to_vec();
                            let new_parsed = stmts()(new_tail);
                            match new_parsed {
                                Ok(new_parsed) => {
                                    for (head, tail) in new_parsed {
                                        if tail.is_empty() {
                                            result.insert(head.clone());
                                        }
                                    }
                                }
                                Err(e) => return Err(e),
                            }
                        }
                    }
                }
            }
            Ok(result)
        }
        Err(e) => Err(e),
    }
}
#[test]
fn parse_file_test() {
    let tokens = match fs::read_to_string("programs/while_func.rimp") {
        Ok(program) => tokenise(program.clone().as_str()),
        Err(e) => {
            println!("Failed to read file. {}", e);
            return;
        }
    };
    let tokens = match tokens {
        Ok(tokens) => tokens,
        Err(e) => {
            println!("Failed to tokenise program. {}", e);
            return;
        }
    };
    println!("{:?}", tokens);
    let result = parse_program(tokens);
    println!("{:?}", result);
}
