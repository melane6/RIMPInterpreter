use crate::regex::Regex::{Char, Range};
use crate::regex::{der, nullable, Regex};
use log::debug;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    Empty,
    Chr(char),
    Seq(Box<Val>, Box<Val>),
    Left(Box<Val>),
    Right(Box<Val>),
    Stars(Vec<Box<Val>>),
    Rec(String, Box<Val>),
    Range(Vec<Box<Val>>),
    // Takes in a vector of values that are in the range
    Plus(Box<Val>),
    // Plus takes in a value
    Optional(Box<Val>),
    UnusedOptional(Box<Val>),
    NTimes(Box<Val>, usize),
}

// extracts a string from a value
pub fn flatten(v: &Val) -> String {
    // println!("flatten v: {:?}", v);
    match v {
        Val::Empty => "".to_string(),
        Val::Chr(c) => c.to_string(),
        Val::Seq(v1, v2) => flatten(v1) + &flatten(v2),
        Val::Left(v) => flatten(v),
        Val::Right(v) => flatten(v),
        Val::Stars(v) => {
            let mut s = "".to_string();
            for v in v {
                s += &flatten(v);
            }
            s
        }
        Val::Range(v) => {
            let mut s = "".to_string();
            for v in v {
                s += &flatten(v);
            }
            s
        }
        Val::Rec(_, v) => flatten(v),
        Val::Plus(v) => flatten(v),
        Val::Optional(v) => flatten(v),
        Val::UnusedOptional(v) => flatten(v),
        Val::NTimes(v, n) => {
            let mut s = "".to_string();
            for _ in 0..*n {
                s += &flatten(v);
            }
            s
        }
    }
}

// extracts an environment from a value;
pub fn env(v: &Val) -> Vec<(String, String)> {
    match v {
        Val::Empty => vec![],
        Val::Chr(_c) => vec![],
        Val::Seq(v1, v2) => [env(v1), env(v2)].concat(),
        Val::Left(v) => env(v),
        Val::Right(v) => env(v),
        Val::Stars(vs) => {
            let mut envs = vec![];
            for v in vs {
                let value = env(&v);
                envs = [envs, value].concat();
            }
            return envs;
        }
        Val::Range(vs) => {
            let mut envs = vec![];
            for v in vs {
                let value = env(&v);
                envs = [envs, value].concat();
            }
            return envs;
        }
        Val::Rec(x, v) => {
            let mut envs = vec![];
            let record = (x.clone(), flatten(v));
            let value = env(&v);
            envs.push(record);
            envs = [envs, value].concat();
            return envs;
        }
        Val::Plus(v) => env(v),
        Val::Optional(v) => env(v),
        Val::UnusedOptional(_v) => vec![],
        Val::NTimes(v, n) => {
            let mut envs = vec![];
            for _ in 0..*n {
                let value = env(&v);
                envs = [envs, value].concat();
            }
            return envs;
        }
    }
}

pub fn mkeps(r: &Regex) -> Val {
    match r {
        Regex::One => Val::Empty,
        Regex::Seq(r1, r2) => Val::Seq(Box::new(mkeps(&r1)), Box::new(mkeps(&r2))),
        Regex::Alt(r1, r2) => {
            if nullable(r1) {
                Val::Left(Box::new(mkeps(&r1)))
            } else {
                Val::Right(Box::new(mkeps(&r2)))
            }
        }
        Regex::Star(_r) => Val::Stars(vec![]),
        Range(_r) => Val::Range(vec![]),
        Regex::Recd(x, r) => Val::Rec(x.clone(), Box::new(mkeps(&r))),
        Regex::Plus(r) => {
            if nullable(r) {
                Val::Plus(Box::new(mkeps(&r)))
            } else {
                Val::Empty
            }
        }
        Regex::Optional(r) => {
            if nullable(r) {
                Val::Optional(Box::new(mkeps(&r)))
            } else {
                Val::UnusedOptional(Box::new(Val::Empty))
            }
        }
        Regex::Ntimes(r, n) => {
            let mut vec = vec![];
            for _ in 0..*n {
                vec.push(Box::new(mkeps(&r)));
            }
            return Val::Stars(vec);
        }
        _ => panic!("mkeps error {:?}", r),
    }
}

pub fn inj(r: &Regex, c: &char, v: &Val) -> Val {
    // println!("c: {} r: {:?} v: {:?}", c, r, v);
    match (r, v) {
        (Regex::Star(r), Val::Seq(v1, v2)) => match *v2.clone() {
            Val::Stars(vs) => {
                let mut injection = vec![];
                let value = inj(&r, &c, v1);
                injection.push(Box::new(value));
                for v in vs {
                    injection.push(v);
                }
                return Val::Stars(injection);
            }
            _ => panic!("inj error"),
        },
        (Regex::Seq(r1, _r2), Val::Seq(v1, v2)) => {
            let val = inj(&r1, c, v1);
            Val::Seq(Box::new(val), Box::new(*v2.clone()))
        }
        (Regex::Seq(r1, _r2), Val::Left(sequence)) => match *sequence.clone() {
            Val::Seq(v1, v2) => Val::Seq(Box::new(inj(&r1, c, &v1)), v2),
            _ => panic!("inj error"),
        },
        (Regex::Seq(r1, r2), Val::Right(v2)) => {
            Val::Seq(Box::new(mkeps(r1)), Box::new(inj(r2, c, v2)))
        }
        (Regex::Alt(r1, _r2), Val::Left(v1)) => Val::Left(Box::new(inj(r1, c, v1))),
        (Regex::Alt(_r1, r2), Val::Right(v2)) => Val::Right(Box::new(inj(r2, c, v2))),
        (Regex::Char(_d), Val::Empty) => Val::Chr(*c),
        (Regex::Range(_r), Val::Empty) => Val::Chr(*c),
        (Regex::Recd(x, r1), _) => Val::Rec(x.clone(), Box::new(inj(r1, c, v))),
        (Regex::Plus(r), Val::Seq(v1, v2)) => {
            let val = inj(&r, c, v1);
            Val::Seq(Box::new(val), Box::new(*v2.clone()))
        }
        (Regex::Optional(r), Val::Empty) => {
            let val = inj(&r, c, &Val::Empty);
            Val::Optional(Box::new(val))
        }
        (Regex::Optional(r), Val::UnusedOptional(v)) => {
            println!("unsied optional");
            let val = inj(&r, c, v);
            Val::Optional(Box::new(val))
        }
        (Regex::Ntimes(r, _n), Val::Seq(v1, v2)) => match *v2.clone() {
            Val::Stars(vs) => {
                let mut injection = vec![];
                let value = inj(&r, &c, v1);
                injection.push(Box::new(value));
                for v in vs {
                    injection.push(v.clone());
                }
                return Val::Stars(injection);
            }
            _ => panic!("inj error"),
        },
        _ => panic!("Panic at inj with the regex: {:?}, value: {:?}", r, v),
    }
}

pub fn f_right(f: Box<dyn Fn(Box<Val>) -> Box<Val>>) -> Box<dyn Fn(Box<Val>) -> Box<Val>> {
    Box::new(move |v| Box::new(Val::Right(f(v))))
}

pub fn f_left(f: Box<dyn Fn(Box<Val>) -> Box<Val>>) -> Box<dyn Fn(Box<Val>) -> Box<Val>> {
    Box::new(move |v| Box::new(Val::Left(f(v))))
}

pub fn f_seq(
    f1: Box<dyn Fn(Box<Val>) -> Box<Val>>,
    f2: Box<dyn Fn(Box<Val>) -> Box<Val>>,
) -> Box<dyn Fn(Box<Val>) -> Box<Val>> {
    Box::new(move |v| match *v {
        Val::Seq(v1, v2) => Box::new(Val::Seq(f1(v1), f2(v2))),
        _ => panic!("f_seq error"),
    })
}

pub fn f_seq_empty1(
    f1: Box<dyn Fn(Box<Val>) -> Box<Val>>,
    f2: Box<dyn Fn(Box<Val>) -> Box<Val>>,
) -> Box<dyn Fn(Box<Val>) -> Box<Val>> {
    Box::new(move |v| Box::new(Val::Seq(f1(Box::new(Val::Empty)), f2(v))))
}

pub fn f_seq_empty2(
    f1: Box<dyn Fn(Box<Val>) -> Box<Val>>,
    f2: Box<dyn Fn(Box<Val>) -> Box<Val>>,
) -> Box<dyn Fn(Box<Val>) -> Box<Val>> {
    Box::new(move |v| Box::new(Val::Seq(f1(v), f2(Box::new(Val::Empty)))))
}

pub fn f_alt(
    f1: Box<dyn Fn(Box<Val>) -> Box<Val>>,
    f2: Box<dyn Fn(Box<Val>) -> Box<Val>>,
) -> Box<dyn Fn(Box<Val>) -> Box<Val>> {
    Box::new(move |v| {
        Box::from({
            match *v {
                Val::Left(v1) => Val::Left(f1(Box::new(*v1))),
                Val::Right(v2) => Val::Right(f2(Box::new(*v2))),
                _ => panic!("f_alt error"),
            }
        })
    })
}

pub fn f_error() -> Val {
    panic!("f_error")
}

pub fn modified_simp(r: &Regex) -> (Regex, Box<dyn Fn(Box<Val>) -> Box<Val>>) {
    match r {
        Regex::Alt(r1, r2) => {
            let (r1s, f1s) = modified_simp(r1);
            let (r2s, f2s) = modified_simp(r2);
            match (r1s, r2s) {
                (Regex::Zero, r2s) => (r2s, f_right(f2s)),
                (r1s, Regex::Zero) => (r1s, f_left(f1s)),
                (r1s, r2s) => {
                    if r1s == r2s {
                        (r1s, f_left(f1s))
                    } else {
                        (Regex::Alt(Box::new(r1s), Box::new(r2s)), f_alt(f1s, f2s))
                    }
                }
            }
        }
        Regex::Seq(r1, r2) => {
            let (r1s, f1s) = modified_simp(r1);
            let (r2s, f2s) = modified_simp(r2);
            // println!("r1s: {:?} r2s: {:?}", r1s, r2s);
            match (r1s, r2s) {
                (Regex::Zero, _) => (Regex::Zero, Box::new(|_x| Box::new(f_error()))), // val error
                (_, Regex::Zero) => (Regex::Zero, Box::new(|_x| Box::new(f_error()))), // val error
                (Regex::One, r2s) => (r2s, f_seq_empty1(f1s, f2s)),
                (r1s, Regex::One) => (r1s, f_seq_empty2(f1s, f2s)),
                (r1s, r2s) => (Regex::Seq(Box::new(r1s), Box::new(r2s)), f_seq(f1s, f2s)),
            }
        }
        _ => (r.clone(), Box::new(|v| v)),
    }
}

#[derive(Debug, PartialEq)]
pub enum LexingError {
    MatchingError,
}

pub type LexResult = Result<Val, LexingError>;

pub fn lex(s: &str, r: Regex) -> LexResult {
    match s.chars().next() {
        None => {
            if nullable(&r) {
                Ok(mkeps(&r))
            } else {
                debug!(" Matching Error :{:?}", r);
                Err(LexingError::MatchingError)
            }
        }
        Some(c) => {
            // println!("started char: {}", c);
            let (r_simp, f_simp) = modified_simp(&der(&c, &r));
            match lex(&s[1..], r_simp) {
                Ok(v) => Ok(inj(&r, &c, &f_simp(Box::new(v)))),
                Err(x) => Err(x),
            }
        }
    }
}

pub fn lexing(s: &str, r: Regex) -> Result<Vec<(String, String)>, LexingError> {
    match lex(s, r) {
        Ok(v) => Ok(env(&v)),
        Err(x) => Err(x),
    }
}

// RIMP Language Regex
pub enum Language {
    SYM,
    DIGIT,
    LETTERS,
    ID,
    NUM,
    KEYWORD,
    SEMI,
    OP,
    WHITESPACE,
    PARENTHESIS,
    STRING,
    COMMENTS,
    CHARACTERS,
    EndOfLine,
    RIMPRegex,
}

impl Language {
    pub fn regex(self) -> Regex {
        match self {
            Language::SYM => Range(
                ('a'..='z')
                    .chain('A'..='Z')
                    .chain(['_', '.', '<', '>', '=', ';', ',', '\\', ':'])
                    .collect(),
            ),
            Language::DIGIT => Range(('0'..='9').collect()),
            Language::LETTERS => Range(('a'..='z').chain('A'..='Z').collect()),
            Language::ID => Regex::Seq(
                Box::new(Language::LETTERS.regex()),
                Box::new(Regex::Star(Box::new(Regex::alt_rs(
                    &Language::LETTERS.regex(),
                    &Language::DIGIT.regex(),
                )))),
            ),
            Language::NUM => Regex::alt_rs(
                &Char('0'),
                &Regex::Seq(
                    Box::new(Range(('1'..='9').collect())),
                    Box::new(Regex::Star(Box::new(Language::DIGIT.regex()))),
                ),
            ),
            Language::KEYWORD => Regex::alt_r(
                "skip",
                &Regex::alt_r(
                    "while",
                    &Regex::alt_r(
                        "do",
                        &Regex::alt_r(
                            "if",
                            &Regex::alt_r(
                                "else",
                                &Regex::alt_r(
                                    "then",
                                    &Regex::alt_r(
                                        "call",
                                        &Regex::alt_r(
                                            "true",
                                            &Regex::alt_r(
                                                "false",
                                                &Regex::alt_r("for", &Regex::alt("do", "uncall")),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
            Language::SEMI => Char(';'),
            Language::OP => Regex::alt_r(
                ":=",
                &Regex::alt_r(
                    "!=",
                    &Regex::alt_r(
                        "==",
                        &Regex::alt_r(
                            "<=",
                            &Regex::alt_r(
                                ">=",
                                &Regex::alt_r(
                                    "%",
                                    &Regex::alt_r(
                                        "||",
                                        &Regex::alt_r(
                                            "&&",
                                            &Regex::alt_r(
                                                "<",
                                                &Regex::alt_r(
                                                    ">",
                                                    &Regex::alt_r(
                                                        "+",
                                                        &Regex::alt_r("-", &Regex::alt("*", "/")),
                                                    ),
                                                ),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
            Language::WHITESPACE => Regex::alt_rs(
                &Regex::Plus(Box::new(Char(' '))),
                &Regex::alt_r("\n", &Regex::alt_r("\t", &Regex::alt("\r", "\r\n"))),
            ),
            Language::PARENTHESIS => Regex::alt_r(
                "{",
                &Regex::alt_r("}", &Regex::alt_r("(", &Regex::alt("(", ")"))),
            ),
            Language::STRING => Regex::Seq(
                Box::new(Char('"')),
                Box::new(Regex::Seq(
                    Box::new(Regex::Star(Box::new(Regex::alt_rs(
                        &Language::SYM.regex(),
                        &Regex::alt_rs(
                            &Language::DIGIT.regex(),
                            &Regex::alt_rs(
                                &Language::PARENTHESIS.regex(),
                                &Regex::alt_rs(
                                    &Regex::Plus(Box::new(Language::WHITESPACE.regex())),
                                    &Regex::from_str("\n").unwrap(),
                                ),
                            ),
                        ),
                    )))),
                    Box::new(Char('"')),
                )),
            ),
            Language::COMMENTS => Regex::Seq(
                Box::new(Regex::from_str("//").unwrap()),
                Box::new(Regex::Seq(
                    Box::new(Regex::Star(Box::new(Regex::alt_rs(
                        &Language::SYM.regex(),
                        &Regex::alt_rs(
                            &Language::PARENTHESIS.regex(),
                            &Regex::alt_rs(
                                &Language::DIGIT.regex(),
                                &Regex::Plus(Box::new(Char(' '))),
                            ),
                        ),
                    )))),
                    Box::new(Language::EndOfLine.regex()),
                )),
            ),
            Language::CHARACTERS => Regex::Star(Box::new(Regex::alt_rs(
                &Language::SYM.regex(),
                &Regex::alt_rs(&Language::DIGIT.regex(), &Language::PARENTHESIS.regex()),
            ))),
            Language::EndOfLine => Regex::Alt(
                Box::new(Char('\n')),
                Box::new(Regex::from_str("\r\n").unwrap()),
            ),
            Language::RIMPRegex => Regex::Star(Box::new(Regex::alt_rs(
                &Regex::Recd("k".to_string(), Box::new(Language::KEYWORD.regex())),
                &Regex::alt_rs(
                    &Regex::Recd("i".to_string(), Box::new(Language::ID.regex())),
                    &Regex::alt_rs(
                        &Regex::Recd("o".to_string(), Box::new(Language::OP.regex())),
                        &Regex::alt_rs(
                            &Regex::Recd("n".to_string(), Box::new(Language::NUM.regex())),
                            &Regex::alt_rs(
                                &Regex::Recd("s".to_string(), Box::new(Language::SEMI.regex())),
                                &Regex::alt_rs(
                                    &Regex::Recd(
                                        "c".to_string(),
                                        Box::new(Language::COMMENTS.regex()),
                                    ),
                                    &Regex::alt_rs(
                                        &Regex::Recd(
                                            "str".to_string(),
                                            Box::new(Language::STRING.regex()),
                                        ),
                                        &Regex::alt_rs(
                                            &Regex::Recd(
                                                "p".to_string(),
                                                Box::new(Language::PARENTHESIS.regex()),
                                            ),
                                            &Regex::Recd(
                                                "w".to_string(),
                                                Box::new(Language::WHITESPACE.regex()),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ))),
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Token {
    Semi,
    LParent,
    RParent,
    LBracket,
    RBracket,
    Id(String),
    FnId(String),
    Op(Operation),
    Num(i64),
    Keyword(Keyword),
    String(String),
    Comment(String),
    Whitespaces(String),
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum Operation {
    Assign,
    Unassign,
    NotEqual,
    Equal,
    LessThanEqual,
    GreaterThanEqual,
    Mod,
    Or,
    And,
    LessThan,
    GreaterThan,
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Operation::Assign => write!(f, ":="),
            Operation::Unassign => write!(f, "=:"),
            Operation::NotEqual => write!(f, "!="),
            Operation::Equal => write!(f, "=="),
            Operation::LessThanEqual => write!(f, "<="),
            Operation::GreaterThanEqual => write!(f, ">="),
            Operation::Mod => write!(f, "%"),
            Operation::Or => write!(f, "||"),
            Operation::And => write!(f, "&&"),
            Operation::LessThan => write!(f, "<"),
            Operation::GreaterThan => write!(f, ">"),
            Operation::Plus => write!(f, "+"),
            Operation::Minus => write!(f, "-"),
            Operation::Multiply => write!(f, "*"),
            Operation::Divide => write!(f, "/"),
            Operation::Power => write!(f, "^"),
        }
    }
}

impl fmt::Debug for Operation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Operation::Assign => write!(f, ":="),
            Operation::Unassign => write!(f, "=:"),
            Operation::NotEqual => write!(f, "!="),
            Operation::Equal => write!(f, "=="),
            Operation::LessThanEqual => write!(f, "<="),
            Operation::GreaterThanEqual => write!(f, ">="),
            Operation::Mod => write!(f, "%"),
            Operation::Or => write!(f, "||"),
            Operation::And => write!(f, "&&"),
            Operation::LessThan => write!(f, "<"),
            Operation::GreaterThan => write!(f, ">"),
            Operation::Plus => write!(f, "+"),
            Operation::Minus => write!(f, "-"),
            Operation::Multiply => write!(f, "*"),
            Operation::Divide => write!(f, "/"),
            Operation::Power => write!(f, "^"),
        }
    }
}

impl FromStr for Operation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ":=" => Ok(Operation::Assign),
            "!=" => Ok(Operation::NotEqual),
            "==" => Ok(Operation::Equal),
            "<=" => Ok(Operation::LessThanEqual),
            ">=" => Ok(Operation::GreaterThanEqual),
            "%" => Ok(Operation::Mod),
            "||" => Ok(Operation::Or),
            "&&" => Ok(Operation::And),
            "<" => Ok(Operation::LessThan),
            ">" => Ok(Operation::GreaterThan),
            "+" => Ok(Operation::Plus),
            "-" => Ok(Operation::Minus),
            "*" => Ok(Operation::Multiply),
            "/" => Ok(Operation::Divide),
            _ => Err(()),
        }
    }
}
#[derive(Clone, Eq, Hash, PartialEq)]
pub enum Keyword {
    Skip,
    While,
    Do,
    If,
    Else,
    Then,
    True,
    False,
    For,
    Call,
    Fn,
}

impl FromStr for Keyword {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "skip" => Ok(Keyword::Skip),
            "while" => Ok(Keyword::While),
            "do" => Ok(Keyword::Do),
            "if" => Ok(Keyword::If),
            "else" => Ok(Keyword::Else),
            "then" => Ok(Keyword::Then),
            "true" => Ok(Keyword::True),
            "false" => Ok(Keyword::False),
            "for" => Ok(Keyword::For),
            "call" => Ok(Keyword::Call),
            "fn" => Ok(Keyword::Fn),
            _ => Err(()),
        }
    }
}

impl fmt::Debug for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Keyword::Skip => write!(f, "skip"),
            Keyword::While => write!(f, "while"),
            Keyword::Do => write!(f, "do"),
            Keyword::If => write!(f, "if"),
            Keyword::Else => write!(f, "else"),
            Keyword::Then => write!(f, "then"),
            Keyword::True => write!(f, "true"),
            Keyword::False => write!(f, "false"),
            Keyword::For => write!(f, "for"),
            Keyword::Call => write!(f, "call"),
            Keyword::Fn => write!(f, "fn"),
        }
    }
}

fn token(s1: &str, s2: &str) -> Option<Token> {
    debug!("s1: {} s2: {}", s1, s2);
    match (s1, s2) {
        ("s", ";") => Some(Token::Semi),
        ("p", "{") => Some(Token::LParent),
        ("p", "}") => Some(Token::RParent),
        ("p", "(") => Some(Token::LBracket),
        ("p", ")") => Some(Token::RBracket),
        ("i", s) => Some(Token::Id(s.to_string())),
        ("o", s) => Some(Token::Op(Operation::from_str(s).unwrap())),
        ("n", s) => Some(Token::Num(s.parse().ok().unwrap())),
        ("k", s) => Some(Token::Keyword(Keyword::from_str(s).unwrap())),
        ("str", s) => Some(Token::String(escape(s))),
        ("fn", s) => Some(Token::FnId(s.to_string())),
        _ => None,
    }
}

fn escape(s: &str) -> String {
    let mut escaped = String::new();
    match s {
        "\"\\n\"" => escaped.push_str("\n"),
        "\"\\t\"" => escaped.push_str("\t"),
        "\"\\r\"" => escaped.push_str("\r"),
        _ => escaped.push_str(s),
    }
    escaped
}

pub(crate) fn tokenise(s: &str) -> Vec<Token> {
    let mut tokens = vec![];
    match lexing(s, Language::RIMPRegex.regex()) {
        Ok(records) => {
            for (s1, s2) in records {
                match token(&s1, &s2) {
                    Some(token) => tokens.push(token),
                    _ => {}
                }
            }
        }
        Err(e) => {
            debug!("Error: {:?}", e);
        }
    }
    tokens
}
