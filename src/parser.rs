use crate::lexer::{Keyword, Operation, Token};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use crate::regex::{prefix_matcher, Regex};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ParseError {
    UnexpectedToken(Token),
    SyntaxError(String),
    FunctionNotImplemented(String),
    ExpectedToken(Token),
    EOF
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

type ParseResult<T, I> = Result<HashSet<(T,I)>, ParseError>;

fn parse_all<I, T>(parsed: ParserResult<T, Vec<I>>) -> HashSet<T>
    where
        I: Debug + Clone + PartialEq + Eq + Hash + 'static,
        T: Debug + Clone + PartialEq + Eq + Hash + 'static,
{
    let mut result = HashSet::new();
    for (head, tail) in parsed {
        if tail.is_empty() {
            result.insert(head);
        }
    }
    result
}

fn alt<'a, I, T, Left, Right>(
    left_parser: Left,
    right_parser: Right,
) -> impl Fn(I) -> ParserResult<T, I>
    where
        I: Debug + Clone + PartialEq + Eq + Hash + 'static,
        T: Debug + Clone + PartialEq + Eq + Hash + 'static,
        Left: Fn(I) -> ParserResult<T, I>,
        Right: Fn(I) -> ParserResult<T, I>,
{
    move |ts: I| {
        let mut result = HashSet::new();
        let left_result = left_parser(ts.clone());
        let right_result = right_parser(ts.clone());
        result.extend(left_result);
        result.extend(right_result);
        result
    }
}

fn seq<'a, I, T, S, Left, Right>(
    left_parser: Left,
    right_parser: Right,
) -> impl Fn(I) -> ParserResult<(T, S), I>
    where
        I: Debug + Clone + PartialEq + Eq + Hash + 'static,
        T: Debug + Clone + PartialEq + Eq + Hash + 'static,
        S: Debug + Clone + PartialEq + Eq + Hash + 'static,
        Left: Fn(I) -> ParserResult<T, I>,
        Right: Fn(I) -> ParserResult<S, I>,
{
    move |ts: I| {
        let mut result = HashSet::new();
        let left_result = left_parser(ts.clone());
        for (head, tail) in left_result {
            let right_result = right_parser(tail);
            for (head2, tail2) in right_result {
                result.insert(((head.clone(), head2.clone()), tail2.clone()));
            }
        }
        result
    }
}

fn map<'a, I, T, S, P, F>(parser: P, f: F) -> impl Fn(I) -> ParserResult<S, I>
    where
        I: Debug + Clone + PartialEq + Eq + Hash + 'static,
        T: Debug + Clone + PartialEq + Eq + Hash + 'static,
        S: Debug + Clone + PartialEq + Eq + Hash + 'static,
        P: Fn(I) -> ParserResult<T, I>,
        F: Fn(T) -> S + 'a,
{
    move |ts: I| {
        let mut result = HashSet::new();
        let parser_result = parser(ts.clone());
        for (head, tail) in parser_result {
            result.insert(((f)(head), tail));
        }
        result
    }
}

fn regex_parser(regex: Regex) -> impl Fn(String) -> ParserResult<String, String> {
    move |ts: String| {
        let mut result = HashSet::new();
        for (head, tail) in prefix_matcher(&ts, regex.clone()) {
            result.insert((head.clone(), tail.clone()));
        }
        result
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

    pub fn check_variables(&mut self, program: Block) {
        todo!()
    }

    pub fn check_valid_fn_call(&mut self) {
        todo!()
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
                _ => Err(ParseError::ExpectedToken(Token::Id("Some variable id".parse().unwrap())))
            }
        } else {
            Err(ParseError::EOF)
        }
    }
}

#[test]
fn id_token_parser_test() {
    let tokens = vec![
        Token::Op(":=".to_string().parse().unwrap()),
        Token::Num(1),
    ];
    let result = id_token_parser()(tokens);
    println!("{:?}", result);
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
                _ => Err(ParseError::ExpectedToken(Token::Id("Some function id".parse().unwrap())))
            }
        } else {
            Err(ParseError::EOF)
        }
    }
}

#[test]
fn fn_id_token_parser_test() {
    let tokens = vec![
        Token::Op(":=".to_string().parse().unwrap()),
        Token::Num(1),
    ];
    let result = id_token_parser()(tokens);
    println!("{:?}", result);
}