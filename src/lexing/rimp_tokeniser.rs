use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;
use log::debug;
use crate::lexing::lexer::{lexing, LexingError};
use crate::lexing::regex::{Regex, Regex::Char, Regex::Range};

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
    COMMA,
    CAPITAL,
    GlobalID,
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
            Language::CAPITAL => Range(('A'..='Z').collect()),
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
                                        "fn",
                                        &Regex::alt_r(
                                            "true",
                                            &Regex::alt_r(
                                                "false",
                                                &Regex::alt_r("def", &Regex::alt("do", "return")),
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
            Language::COMMA => Char(','),
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
            Language::GlobalID => Regex::Seq(
                Box::new(Language::CAPITAL.regex()),
                Box::new(Regex::Star(Box::new(Regex::alt_rs(
                    &Language::LETTERS.regex(),
                    &Language::DIGIT.regex(),
                )))),
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
                    &Regex::Recd("gi".to_string(), Box::new(Language::GlobalID.regex())),
                    &Regex::alt_rs(
                        &Regex::Recd("o".to_string(), Box::new(Language::OP.regex())),
                        &Regex::alt_rs(
                            &Regex::Recd("n".to_string(), Box::new(Language::NUM.regex())),
                            &Regex::alt_rs(
                                &Regex::Recd("co".to_string(), Box::new(Language::COMMA.regex())),
                                &Regex::alt_rs(
                                    &Regex::Recd("i".to_string(), Box::new(Language::ID.regex())),
                                    &Regex::alt_rs(
                                        &Regex::Recd(
                                            "s".to_string(),
                                            Box::new(Language::SEMI.regex()),
                                        ),
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
    GlobalId(String),
    FnId(String),
    Op(Operation),
    Num(i64),
    Keyword(Keyword),
    String(String),
    Comment(String),
    Whitespaces(String),
    Comma,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Token::Semi => write!(f, ";"),
            Token::LParent => write!(f, "{{"),
            Token::RParent => write!(f, "}}"),
            Token::LBracket => write!(f, "("),
            Token::RBracket => write!(f, ")"),
            Token::Id(s) => write!(f, "{}", s),
            Token::GlobalId(s) => write!(f, "{}", s),
            Token::FnId(s) => write!(f, "{}", s),
            Token::Op(o) => write!(f, "{}", o.to_string()),
            Token::Num(n) => write!(f, "{}", n),
            Token::Keyword(k) => write!(f, "{}", k.to_string()),
            Token::String(s) => write!(f, "{}", s),
            Token::Comment(s) => write!(f, "{}", s),
            Token::Whitespaces(s) => write!(f, "{}", s),
            Token::Comma => write!(f, ","),
        }
    }
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
    Not,
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
            Operation::Not => write!(f, "!"),
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
            Operation::Not => write!(f, "!"),
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
            "^" => Ok(Operation::Power),
            "!" => Ok(Operation::Not),
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
    Def,
    Return,
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
            "def" => Ok(Keyword::Def),
            "return" => Ok(Keyword::Return),
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
            Keyword::Def => write!(f, "def"),
            Keyword::Return => write!(f, "return"),
        }
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
            Keyword::Def => write!(f, "def"),
            Keyword::Return => write!(f, "return"),
        }
    }
}

fn token(s1: &str, s2: &str) -> Result<Token, LexingError> {
    debug!("s1: {} s2: {}", s1, s2);
    match (s1, s2) {
        ("s", ";") => Ok(Token::Semi),
        ("p", "{") => Ok(Token::LParent),
        ("p", "}") => Ok(Token::RParent),
        ("p", "(") => Ok(Token::LBracket),
        ("p", ")") => Ok(Token::RBracket),
        ("i", s) => Ok(Token::Id(s.to_string())),
        ("o", s) => {
            let op = Operation::from_str(s);
            match op {
                Ok(op) => Ok(Token::Op(op)),
                Err(_) => Err(LexingError::TokenError(s.to_string())),
            }
        },
        ("n", s) => {
            let num = s.parse::<i64>();
            match num {
                Ok(num) => Ok(Token::Num(num)),
                Err(_) => Err(LexingError::TokenError(s.to_string())),
            }
        },
        ("k", s) => {
            let keyword = Keyword::from_str(s);
            match keyword {
                Ok(keyword) => Ok(Token::Keyword(keyword)),
                Err(_) => Err(LexingError::TokenError(s.to_string())),
            }
        },
        ("str", s) => Ok(Token::String(escape(s))),
        ("fn", s) => Ok(Token::FnId(s.to_string())),
        ("co", ",") => Ok(Token::Comma),
        ("gi", s) => Ok(Token::GlobalId(s.to_string())),
        _ => Err(LexingError::TokenError(s1.to_string())),
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

pub(crate) fn tokenise(s: &str) -> Result<Vec<Token>,LexingError> {
    let mut tokens = vec![];
    match lexing(s, Language::RIMPRegex.regex()) {
        Ok(records) => {
            for (s1, s2) in records {
                match token(&s1, &s2) {
                    Ok(token) => tokens.push(token),
                    Err(e) => {
                        debug!("Error: {:?}", e);
                        return Err(e);
                    }
                }
            }
        }
        Err(e) => {
            debug!("Error: {:?}", e);
            return Err(e);
        }
    }
    Ok(tokens)
}

#[test]
fn test_lexing() {
    let file = std::fs::read_to_string("programs/fib.rimp").unwrap();
    let tokens = tokenise(file.as_str());
    println!("tokens: {:?}", tokens)
}
