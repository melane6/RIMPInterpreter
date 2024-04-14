use crate::lexing::regex::{der, nullable, Regex};
use log::debug;
use std::fmt;
use std::fmt::Formatter;
use crate::lexing::regex::Regex::Range;


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
    TokenError(String),
}

impl fmt::Display for LexingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LexingError::MatchingError => write!(f, "Matching Error"),
            LexingError::TokenError(s) => write!(f, "Token Error: {}", s),
        }
    }
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

