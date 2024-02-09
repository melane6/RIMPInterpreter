use crate::lexer::{Keyword, Operation, Token};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;

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
}

pub type ParseError = (ParseErrorMessage, usize);
pub type Block = Vec<Stmt>;

pub type FnArgs = Vec<ArithmeticExpr>;

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
    LocalID(String),
    GlobalID(String),
    Aop(String, Box<ArithmeticExpr>, Box<ArithmeticExpr>),
    Call(String, Vec<ArithmeticExpr>),
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

pub(crate) type ParseResult<T, I> = Result<HashSet<(T, I)>, ParseError>;

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
) -> impl Fn(I) -> ParseResult<T, I>
where
    I: Debug + Clone + PartialEq + Eq + Hash + 'static,
    T: Debug + Clone + PartialEq + Eq + Hash + 'static,
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
            (Err(left_error), Err(right_error)) => Err(left_error),
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
                if result.is_empty() && no_match { Err(error) } else { Ok(result) }
            }
            Err(e) => Err(e),
        }
    }
}

#[test]
fn seq_test() {
    let tokens = vec![Token::Num(1), Token::Op(Operation::Plus),Token::Num(1)];
    let result = seq(number_parser(), seq(op_parser(), number_parser()))(tokens);
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
                _ => Err((ParseErrorMessage::ExpectedToken(Token::Id(
                    "Some variable id".parse().unwrap(),
                )), ts.len())),
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

fn function_id_token_parser() -> impl Fn(Vec<Token>) -> ParseResult<String, Vec<Token>> {
    move |ts: Vec<Token>| {
        let mut result = HashSet::new();
        if ts.clone().first().is_some() {
            match ts.first().clone().unwrap() {
                Token::FnId(x) => {
                    result.insert((x.clone(), ts[1..].to_vec()));
                    Ok(result)
                }
                _ => Err((ParseErrorMessage::ExpectedToken(Token::Id(
                    "Some function id".parse().unwrap(),
                )), ts.len())),
            }
        } else {
            Err((ParseErrorMessage::EOF, ts.len()))
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
                        Err((ParseErrorMessage::ExpectedToken(Token::String(
                            "Expected some symbol".to_string(),
                        )), ts.len()))
                    }
                }
                _ => Err((ParseErrorMessage::ExpectedToken(Token::String(
                    "Expected some symbol".to_string(),
                )), ts.len())),
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
                        Err((ParseErrorMessage::ExpectedToken(Token::Keyword(
                            Keyword::from_str(s.as_str()).unwrap(),
                        )), ts.len()))
                    }
                }
                _ => Err((ParseErrorMessage::ExpectedToken(Token::Keyword(
                    Keyword::from_str(s.as_str()).unwrap(),
                )), ts.len())),
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
                    _ => Err((ParseErrorMessage::ExpectedToken(Token::Keyword(
                        Keyword::Skip,
                    )), ts.len())),
                },
                _ => Err((ParseErrorMessage::ExpectedToken(Token::Keyword(
                    Keyword::Skip,
                )), ts.len())),
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
    move |ts| {
        alt(
            map(
                seq(term(), seq(op_parser(), arithmetic_expr())),
                |(fa, (op, a))| ArithmeticExpr::Aop(op.to_string(), Box::new(fa), Box::new(a)),
            ),
            term(),
        )(ts)
    }
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
    move |ts| {
        map(
            seq(
                id_token_parser(),
                map(
                    seq(parenthesis("(".to_string()), seq(params(), parenthesis(")".to_string()))),
                    |(_, (params, _))| params,
                ),
            ),
            |(id, params)| ArithmeticExpr::Call(id, params),
        )(ts)
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
    move |ts| {
        alt(
            map(
                seq(arithmetic_expr(), seq(comma_parser(), params())),
                |(param, (_, mut params))| {
                    params.insert(0, param);
                    params
                },
            ),
            map(arithmetic_expr(), |param| vec![param]),
        )(ts)
    }
}



#[test]
fn params_test() {
    let tokens = vec![Token::Id("x".to_string()), Token::Comma, Token::Id("y".to_string())];
    let result = params()(tokens);
    println!("{:?}", result);
}

fn factor() -> impl Fn(Vec<Token>) -> ParseResult<ArithmeticExpr, Vec<Token>> {
    // global_id? fn_id? local_id?
    move |ts| {
        alt(
            call(),
            alt(
                map(number_parser(), |num| ArithmeticExpr::Num(num)),
                map(id_token_parser(), |id| ArithmeticExpr::LocalID(id)),
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
    let tokens = vec![
        Token::Num(1),
        Token::Op(Operation::NotEqual),
        Token::Num(1),
    ];
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
    move |ts| {
        map(
            seq(
                keyword_parser("if".to_string()),
                seq(
                    boolean_expr(),
                    seq(
                        keyword_parser("then".to_string()),
                        seq(block(), seq(keyword_parser("else".to_string()), block())),
                    ),
                ),
            ),
            |(_, (b, (_, (a1, (_, a2)))))| Stmt::If(b, a1, a2),
        )(ts)
    }
}

#[test]
fn if_parser_test() {
    let tokens = vec![
        Token::Keyword(Keyword::If),
        Token::Num(1),
        Token::Op(Operation::NotEqual),
        Token::Num(1),
        Token::Keyword(Keyword::Then),
        Token::Keyword(Keyword::Skip),
        Token::Keyword(Keyword::Else),
        Token::Keyword(Keyword::Skip),
    ];
    let result = if_parser()(tokens);
    println!("{:?}", result);
}

fn while_parser() -> impl Fn(Vec<Token>) -> ParseResult<Stmt, Vec<Token>> {
    move |ts| {
        map(
            seq(
                keyword_parser("while".to_string()),
                seq(
                    boolean_expr(),
                    seq(keyword_parser("do".to_string()),
                        seq(
                            parenthesis("{".to_string()),
                            seq(
                                block(), parenthesis("}".to_string())
                            )
                        ),
                    ),
                ),
            ),
            |(_, (b, (_, (_, (block, _ )))))| Stmt::While(b, block),
        )(ts)
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
    move |ts| {
        alt(
            map(
                seq(stmt(), seq(semi_colon_parser(), block())),
                |(stmt, (_, mut block))| {
                    block.insert(0, stmt);
                    block
                },
            ),
            map(stmt(), |stmt| vec![stmt]),
        )(ts)
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

fn stmt() -> impl Fn(Vec<Token>) -> ParseResult<Stmt, Vec<Token>> {
    move |ts| {
        alt(
            skip_parser(),
            alt(
                if_parser(),
                alt(
                    while_parser(),
                    alt(
                        map(
                            seq(
                                id_token_parser(),
                                seq(symbol_parser(":=".to_string()), arithmetic_expr()),
                            ),
                            |(id, (_, a))| Stmt::Assign(id, a),
                        ),
                        map(
                            seq(
                                id_token_parser(),
                                seq(symbol_parser("!=".to_string()), arithmetic_expr()),
                            ),
                            |(id, (_, a))| Stmt::Unassign(id, a),
                        ),
                    ),
                ),
            ),
        )(ts)
    }
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
                    _ => Err((ParseErrorMessage::ExpectedToken(Token::Keyword(
                        Keyword::Def,
                    )), vec.len())),
                },
                _ => Err((ParseErrorMessage::ExpectedToken(Token::Keyword(
                    Keyword::from_str("Expected keyword fn").unwrap(),
                )), vec.len())),
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

// Function parser is a parser that parses a function
// definition in the form of:
// fn <function_id>(<params>) { <block> }
fn function_parser() -> impl Fn(Vec<Token>) -> ParseResult<Stmt, Vec<Token>> {
    move |ts| {
        map(
            seq(
               function_id(),
                seq(
                    map(
                        seq(parenthesis("(".to_string()), seq(params(), parenthesis(")".to_string()))),
                        |(_, (params, _))| params,
                    ),
                    seq(
                        parenthesis("{".to_string()),
                        seq(block(), parenthesis("}".to_string())),
                    ),
                ),
            ),
            |(id,(args, (_, (block, _))))| Stmt::Function(id.to_string(), args, block),
        )(ts)
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
        Token::Keyword(Keyword::Skip),
        Token::Semi,
        Token::Keyword(Keyword::Skip),
        Token::RParent,
    ];
    let result = function_parser()(tokens);
    println!("{:?}", result);
}

