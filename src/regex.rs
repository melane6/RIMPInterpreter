use core::fmt;
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Clone, PartialEq)]
pub enum Regex {
    Zero,
    One,
    Char(char),
    Alt(Box<Regex>, Box<Regex>),
    Seq(Box<Regex>, Box<Regex>),
    Star(Box<Regex>),
    Ntimes(Box<Regex>, i32),
    Range(Vec<char>),
    Plus(Box<Regex>),
    Optional(Box<Regex>),
    And(Box<Regex>, Box<Regex>),
    Upto(Box<Regex>, i32),
    From(Box<Regex>, i32),
    Between(Box<Regex>, i32, i32),
    Not(Box<Regex>),
    Cfun(fn(char) -> bool),
    Recd(String, Box<Regex>)
}

fn charlist2rexp(s: &str) -> Regex {
    // convert a string of characters into a regex of just sequence of characters
    let mut r = Regex::Char(s.chars().last().unwrap());
    // remove last character from string
    let s = &s[..s.len() - 1];
    for c in s.chars().rev(){
        r = Regex::Seq(Box::new(Regex::Char(c)), Box::new(r));
    }
    r
}

impl Regex {
    pub fn alt(s1: &str, s2: &str) -> Regex {
        Regex::Alt(Box::new(charlist2rexp(s1)),Box::new(charlist2rexp(s2)))
    }
    pub fn alt_r(s1: &str, r: &Regex) -> Regex {
        Regex::Alt(Box::new(charlist2rexp(s1)),Box::new(r.clone()))
    }

    pub fn alt_rs(r1: &Regex,r2: &Regex) -> Regex {
        Regex::Alt(Box::new(r1.clone()),Box::new(r2.clone()))
    }
}

impl FromStr for Regex {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(charlist2rexp(s))
    }
}


// implement print for debugging
impl fmt::Debug for Regex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Regex::Zero => write!(f, "0"),
            Regex::One => write!(f, "1"),
            Regex::Char(c) => write!(f, "{}", c),
            Regex::Alt(r1, r2) => write!(f, "({:?}|{:?})", r1, r2),
            Regex::Seq(r1, r2) => write!(f, "({:?}{:?})", r1, r2),
            Regex::Star(r) => write!(f, "({:?})*", r),
            Regex::Ntimes(r, n) => write!(f, "({:?}^{})", r, n),
            Regex::Range(l) => {
                let mut s = String::new();
                for c in l {
                    s.push(*c);
                }
                write!(f, "[{}]", s)
            }
            Regex::Plus(r) => write!(f, "({:?})+", r),
            Regex::Optional(r) => write!(f, "({:?})?", r),
            Regex::And(r1, r2) => write!(f, "{:?}&{:?}", r1, r2),
            Regex::Upto(r, n) => write!(f, "({:?}^..{})", r, n),
            Regex::From(r, n) => write!(f, "({:?}^{}..)", r, n),
            Regex::Between(r, n, m) => write!(f, "({:?}^({}..{}))", r, n, m),
            Regex::Not(r) => write!(f, "!{:?}", r),
            Regex::Cfun(_r) => write!(f, "cfun"),
            Regex::Recd(x, r) => write!(f,"!{:?}: {}", r, x ),
        }
    }
}

pub fn nullable(r: &Regex) -> bool {
    match r {
        Regex::Zero => false,
        Regex::One => true,
        Regex::Char(_) => false,
        Regex::Seq(r1, r2) => nullable(&r1) && nullable(&r2),
        Regex::Alt(r1, r2) => nullable(&r1) || nullable(&r2),
        Regex::Star(_r) => true,
        Regex::Ntimes(r, n) => *n == 0 || nullable(&r),
        Regex::Range(_r) => false,
        Regex::Plus(r) => nullable(&r),
        Regex::Optional(_r) => true,
        Regex::And(r1, r2) => nullable(&r1) && nullable(&r2),
        Regex::Upto(_r, _n) => true,
        Regex::From(r, n) => *n == 0 || nullable(&r),
        Regex::Between(r, n, _m) => *n == 0 || nullable(&r),
        Regex::Not(r) => !nullable(&r),
        Regex::Cfun(_) => false,
        Regex::Recd(_, r1) => nullable(&r1),
    }
}


pub fn der(d: &char, r: &Regex) -> Regex {
    match r {
        Regex::Zero => Regex::Zero,
        Regex::One => Regex::Zero,
        Regex::Char(c) => {
            if *c == *d {
                Regex::One
            } else {
                Regex::Zero
            }
        }
        Regex::Alt(r1, r2) => Regex::Alt(Box::new(der(d, &r1)), Box::new(der(d, &r2))),
        Regex::Seq(r1, r2) => {
            if nullable(&r1) {
                Regex::Alt(
                    Box::new(Regex::Seq(Box::new(der(d, &r1)), r2.clone())),
                    Box::new(der(d, &r2)),
                )
            } else {
                Regex::Seq(Box::new(der(d, &r1)), r2.clone())
            }
        }
        Regex::Star(r) => Regex::Seq(Box::new(der(d, &r)), Box::new(Regex::Star(r.clone()))),
        Regex::Ntimes(r, n) => {
            if *n == 0 {
                Regex::Zero
            } else {
                Regex::Seq(
                    Box::new(der(d, &r)),
                    Box::new(Regex::Ntimes(r.clone(), n - 1)),
                )
            }
        }
        Regex::Range(l) => {
            if l.contains(d) {
                Regex::One
            } else {
                Regex::Zero
            }
        }
        Regex::Plus(r) => Regex::Seq(Box::new(der(d, &r)), Box::new(Regex::Star(r.clone()))),
        Regex::Optional(r) => der(d, &r),
        Regex::Upto(r, n) => {
            if *n == 0 {
                Regex::Zero
            } else {
                Regex::Seq(
                    Box::new(der(d, &r)),
                    Box::new(Regex::Upto(r.clone(), n - 1)),
                )
            }
        }
        Regex::From(r, n) => {
            if *n == 0 {
                Regex::Seq(Box::new(der(d, &r)), Box::new(Regex::Star(r.clone())))
            } else {
                Regex::Seq(
                    Box::new(der(d, &r)),
                    Box::new(Regex::From(r.clone(), *n - 1)),
                )
            }
        }
        Regex::Between(r, n, m) => {
            if *n == 0 && *m <= 0 {
                Regex::Zero
            } else if *n == 0 {
                Regex::Seq(
                    Box::new(der(d, &r)),
                    Box::new(Regex::Between(r.clone(), *n, *m - 1)), // CHANGE TO STAR (R)
                )
            } else {
                Regex::Seq(
                    Box::new(der(d, &r)),
                    Box::new(Regex::Between(r.clone(), *n - 1, *m - 1)),
                )
            }
        }
        Regex::Not(r) => Regex::Not(Box::new(der(d, &r))),
        Regex::And(r1, r2) => Regex::And(Box::new(der(d, &r1)), Box::new(der(d, &r2))),
        Regex::Cfun(f) => {
            if f(*d) {
                Regex::One
            } else {
                Regex::Zero
            }
        }
        Regex::Recd(_, r1) => der(d, &r1),
    }
}

pub fn simp(r: &Regex) -> Regex {
    match r {
        Regex::Alt(r1, r2) => {
            let r1s = simp(r1);
            let r2s = simp(r2);
            match (r1s, r2s) {
                (Regex::Zero, r2s) => r2s,
                (r1s, Regex::Zero) => r1s,
                (r1s, r2s) => {
                    if r1s == r2s {
                        r1s
                    } else {
                        Regex::Alt(Box::new(r1s), Box::new(r2s))
                    }
                }
            }
        }
        Regex::Seq(r1, r2) => {
            let r1s = simp(r1);
            let r2s = simp(r2);
            match (r1s, r2s) {
                (Regex::Zero, _) => Regex::Zero,
                (_, Regex::Zero) => Regex::Zero,
                (Regex::One, r2s) => r2s,
                (r1s, Regex::One) => r1s,
                (r1s, r2s) => Regex::Seq(Box::new(r1s), Box::new(r2s)),
            }
        }
        r => r.clone(),
    }
}

pub fn ders(s: &str, r: &Regex) -> Regex {
    match s.chars().next() {
        None => simp(r),
        Some(c) => ders(&s[1..], &simp(&der(&c, &r)))
    }
}

pub fn matcher(s: &str, r: Regex) -> bool {
    nullable(&ders(s, &r))
}

pub fn prefix_matcher(s: &str, r: Regex) -> HashSet<(String,String)> {
    let mut result = HashSet::new();
    let mut s = s.to_string();
    let mut remaining = "".to_string();
    if r == Regex::One {
        result.insert((s.clone(), remaining.clone()));
    }
    while !s.is_empty() {
        if matcher(&s, r.clone()) {
            result.insert((s.clone(), remaining.clone()));
        }
        remaining = s.pop().unwrap().to_string() + &remaining;
    }
    result

}
#[cfg(test)]
mod test {
    use super::{matcher,prefix_matcher, Regex};

    #[test]
    fn test_prefix_matcher() {
        let r = Regex::Seq(Box::new(Regex::One), Box::new(Regex::Char('b')));
        let result = prefix_matcher("a", r);
        println!("{:?}", result);
    }

    #[test]
    fn basic_testcases() {
        assert!(matcher(
            "ab",
            Regex::Seq(Box::new(Regex::Char('a')), Box::new(Regex::Char('b'))),
        )); // "ab" ab True
        assert!(matcher(
            "a",
            Regex::Alt(Box::new(Regex::Char('a')), Box::new(Regex::Char('b'))),
        )); // "a" a|b True
        assert!(matcher(
            "b",
            Regex::Alt(Box::new(Regex::Char('a')), Box::new(Regex::Char('b'))),
        )); // "b" a|b True
        // test case for between 2 and 2 a's with two plus operators
        assert!(matcher(
            "aaaa",
            Regex::Plus(Box::new(Regex::Between(Box::new(Regex::Char('a')), 2, 2))),
        )); // "aaaa" (a{2,2})+ True
        assert!(!matcher(
            "aaaaaaa",
            Regex::Plus(Box::new(Regex::Plus(Box::new(Regex::Between(Box::new(Regex::Char('a')), 2, 2)))
            )))); // "aaaaaaa" ((a{2,2})+)+ False
    }

    #[test]
    fn question3() {
        assert!(matcher("", Regex::Optional(Box::new(Regex::Char('a'))))); // "" a? True
        assert!(matcher("", Regex::Not(Box::new(Regex::Char('a'))))); // "" !a True
        assert!(matcher(
            "aaaaaaa",
            Regex::Ntimes(Box::new(Regex::Char('a')), 7),
        )); // "aaaaaaa" a^7 True
        assert!(matcher(
            "aaaaa",
            Regex::Between(Box::new(Regex::Char('a')), 0, 8),
        )); // "aaaaa" a^(0..8) True
        assert!(matcher("aa", Regex::From(Box::new(Regex::Char('a')), 2))); // "aa" a^2.. True
        assert!(matcher("aaaa", Regex::From(Box::new(Regex::Char('a')), 2))); // "aaaa" a^2.. True
        assert!(matcher("aa", Regex::Upto(Box::new(Regex::Char('a')), 5))); // "aa" a^..5 True
        assert!(matcher("", Regex::Upto(Box::new(Regex::Char('a')), 5))); // "" a^..5 True
        assert!(matcher("aaaaa", Regex::Upto(Box::new(Regex::Char('a')), 5))); // "aaaaa" a^..5 True
    }

    #[test]
    fn and_regex() {
        assert!(!matcher("a", Regex::And(Box::new(Regex::Char('a')), Box::new(Regex::Char('b'))))); // "a" a&b False
        assert!(matcher("a", Regex::And(Box::new(Regex::Char('a')), Box::new(Regex::Star(Box::new(Regex::Char('a'))))))); // "a" a&(a*) True
        assert!(!matcher("aa", Regex::And(Box::new(Regex::Char('a')), Box::new(Regex::Star(Box::new(Regex::Char('b'))))))); // "aa" a&(b*) False
        assert!(!matcher("aa", Regex::And(Box::new(Regex::Char('a')), Box::new(Regex::Star(Box::new(Regex::Char('a'))))))); // "aa" a&(a*) False
    }

    #[test]
    fn optional_regex() {
        assert!(matcher("", Regex::Optional(Box::new(Regex::Char('a')))));
        assert!(matcher("a", Regex::Optional(Box::new(Regex::Char('a')))));
        assert!(!matcher("aa", Regex::Optional(Box::new(Regex::Char('a')))));
        assert!(!matcher("aaa", Regex::Optional(Box::new(Regex::Char('a')))));
        assert!(!matcher(
            "aaaa",
            Regex::Optional(Box::new(Regex::Char('a'))),
        ));
        assert!(!matcher(
            "aaaaa",
            Regex::Optional(Box::new(Regex::Char('a'))),
        ));
        assert!(!matcher(
            "aaaaaa",
            Regex::Optional(Box::new(Regex::Char('a'))),
        ));
    }

    #[test]
    fn not_regex() {
        assert!(matcher("", Regex::Not(Box::new(Regex::Char('a')))));
        assert!(!(matcher("a", Regex::Not(Box::new(Regex::Char('a'))))));
        assert!(matcher("aa", Regex::Not(Box::new(Regex::Char('a')))));
        assert!(matcher("aaa", Regex::Not(Box::new(Regex::Char('a')))));
        assert!(matcher("aaaa", Regex::Not(Box::new(Regex::Char('a')))));
        assert!(matcher("aaaaa", Regex::Not(Box::new(Regex::Char('a')))));
        assert!(matcher("aaaaaa", Regex::Not(Box::new(Regex::Char('a')))));
    }

    #[test]
    fn three_times_regex() {
        assert!(!matcher("", Regex::Ntimes(Box::new(Regex::Char('a')), 3)));
        assert!(!(matcher("a", Regex::Ntimes(Box::new(Regex::Char('a')), 3))));
        assert!(!matcher("aa", Regex::Ntimes(Box::new(Regex::Char('a')), 3)));
        assert!(matcher("aaa", Regex::Ntimes(Box::new(Regex::Char('a')), 3)));
        assert!(!matcher(
            "aaaa",
            Regex::Ntimes(Box::new(Regex::Char('a')), 3),
        ));
        assert!(!matcher(
            "aaaaa",
            Regex::Ntimes(Box::new(Regex::Char('a')), 3),
        ));
        assert!(!matcher(
            "aaaaaa",
            Regex::Ntimes(Box::new(Regex::Char('a')), 3),
        ));
    }

    #[test]
    fn three_times_regex_and_optional_a() {
        assert!(matcher(
            "",
            Regex::Ntimes(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3),
        ));
        assert!(
            (matcher(
                "a",
                Regex::Ntimes(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3),
            ))
        );
        assert!(matcher(
            "aa",
            Regex::Ntimes(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3),
        ));
        assert!(matcher(
            "aaa",
            Regex::Ntimes(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3),
        ));
        assert!(!matcher(
            "aaaa",
            Regex::Ntimes(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3),
        ));
        assert!(!matcher(
            "aaaaa",
            Regex::Ntimes(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3),
        ));
        assert!(!matcher(
            "aaaaaa",
            Regex::Ntimes(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3),
        ));
    }

    #[test]
    fn upto_three_times_on_optional_a() {
        assert!(matcher(
            "",
            Regex::Upto(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3),
        ));
        assert!(
            (matcher(
                "a",
                Regex::Upto(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3),
            ))
        );
        assert!(matcher(
            "aa",
            Regex::Upto(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3),
        ));
        assert!(matcher(
            "aaa",
            Regex::Upto(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3),
        ));
        assert!(!matcher(
            "aaaa",
            Regex::Upto(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3),
        ));
        assert!(!matcher(
            "aaaaa",
            Regex::Upto(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3),
        ));
        assert!(!matcher(
            "aaaaaa",
            Regex::Upto(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3),
        ));
    }

    #[test]
    fn upto_three_times_on_a() {
        assert!(matcher("", Regex::Upto(Box::new(Regex::Char('a')), 3)));
        assert!((matcher("a", Regex::Upto(Box::new(Regex::Char('a')), 3))));
        assert!(matcher("aa", Regex::Upto(Box::new(Regex::Char('a')), 3)));
        assert!(matcher("aaa", Regex::Upto(Box::new(Regex::Char('a')), 3)));
        assert!(!matcher("aaaa", Regex::Upto(Box::new(Regex::Char('a')), 3)));
        assert!(!matcher(
            "aaaaa",
            Regex::Upto(Box::new(Regex::Char('a')), 3),
        ));
        assert!(!matcher(
            "aaaaaa",
            Regex::Upto(Box::new(Regex::Char('a')), 3),
        ));
    }

    #[test]
    fn between_three_and_five_a() {
        assert!(!matcher(
            "",
            Regex::Between(Box::new(Regex::Char('a')), 3, 5),
        ));
        assert!(!(matcher("a", Regex::Between(Box::new(Regex::Char('a')), 3, 5))));
        assert!(!matcher(
            "aa",
            Regex::Between(Box::new(Regex::Char('a')), 3, 5),
        ));
        assert!(matcher(
            "aaa",
            Regex::Between(Box::new(Regex::Char('a')), 3, 5),
        ));
        assert!(matcher(
            "aaaa",
            Regex::Between(Box::new(Regex::Char('a')), 3, 5),
        ));
        assert!(matcher(
            "aaaaa",
            Regex::Between(Box::new(Regex::Char('a')), 3, 5),
        ));
        assert!(!matcher(
            "aaaaaa",
            Regex::Between(Box::new(Regex::Char('a')), 3, 5),
        ));
    }

    #[test]
    fn between_three_and_five_on_optional_a() {
        assert!(matcher(
            "",
            Regex::Between(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3, 5),
        ));
        assert!(
            (matcher(
                "a",
                Regex::Between(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3, 5),
            ))
        );
        assert!(matcher(
            "aa",
            Regex::Between(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3, 5),
        ));
        assert!(matcher(
            "aaa",
            Regex::Between(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3, 5),
        ));
        assert!(matcher(
            "aaaa",
            Regex::Between(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3, 5),
        ));
        assert!(matcher(
            "aaaaa",
            Regex::Between(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3, 5),
        ));
        assert!(!matcher(
            "aaaaaa",
            Regex::Between(Box::new(Regex::Optional(Box::new(Regex::Char('a')))), 3, 5),
        ));
    }

    #[test]
    fn cfun_types() {
        assert!(matcher("a", Regex::Cfun(|_r| true))); // "a" ALL True
        assert!(matcher("a", Regex::Cfun(|c| c == 'a'))); // "a" Char("a") True
        assert!(matcher("d", Regex::Cfun(|c| "asdef".contains(c)))) // "d" Range("asdef".toList) True
    }

    #[test]
    fn email_testcases() {
        fn first_part_email(c: char) -> bool {
            let digits = "0123456789";
            let symbols = "_.-";
            return c.is_lowercase() || digits.contains(c) || symbols.contains(c);
        }
        fn second_part_email(c: char) -> bool {
            let digits = "0123456789";
            let symbols = "_.";
            return c.is_lowercase() || digits.contains(c) || symbols.contains(c);
        }
        fn last_part_email(c: char) -> bool {
            let symbols1 = ".";
            return c.is_lowercase() || symbols1.contains(c);
        }
        let email_part_1 = Regex::Cfun(first_part_email);
        let email_part_2 = Regex::Cfun(second_part_email);
        let email_part_3 = Regex::Cfun(last_part_email);

        let email_regex = Regex::Seq(
            Box::new(Regex::Plus(Box::new(email_part_1))),
            Box::new(Regex::Seq(
                Box::new(Regex::Char('@')),
                Box::new(Regex::Seq(
                    Box::new(Regex::Plus(Box::new(email_part_2))),
                    Box::new(Regex::Seq(
                        Box::new(Regex::Char('.')),
                        Box::new(Regex::Between(Box::new(email_part_3), 2, 6)),
                    )),
                )),
            )),
        );
        assert!(!matcher("melane.navaratnarajah@kcl", email_regex.clone())); // False
        assert!(!matcher("h@.com", email_regex.clone())); // False
        assert!(matcher(
            "melane.navaratnarajah@kcl.ac.uk",
            email_regex.clone(),
        )); // True
        assert!(matcher("k21030747@kcl.ac.uk", email_regex)); // True
    }

    #[test]
    fn question6() {
        // / · ∗ · (∼ (ALL∗ · ∗ · / · ALL∗)) · ∗ · /
        let start = Box::new(Regex::Seq(
            Box::new(Regex::Char('/')),
            Box::new(Regex::Char('*')),
        ));
        let end = Box::new(Regex::Seq(
            Box::new(Regex::Char('*')),
            Box::new(Regex::Char('/')),
        ));
        let all = Box::new(Regex::Cfun(|_r| true));
        let regex = Regex::Seq(
            start,
            Box::new(Regex::Seq(
                Box::new(Regex::Not(Box::new(Regex::Seq(
                    Box::new(Regex::Star(all.clone())),
                    Box::new(Regex::Seq(end.clone(), Box::new(Regex::Star(all)))),
                )))),
                end,
            )),
        );
        println!("{:?}", regex);
        assert!(matcher("/**/", regex.clone())); // True
        assert!(matcher("/*foobar*/", regex.clone())); // True
        assert!(!matcher("/*test*/test*/", regex.clone())); // False
        assert!(matcher("/*test/*test*/", regex.clone())); // True
        assert!(!matcher("testtest*/test*/", regex.clone())); // False
    }

    #[test]
    fn question7() {
        let r1 = Regex::Plus(Box::new(Regex::Plus(Box::new(Regex::Seq(
            Box::new(Regex::Char('a')),
            Box::new(Regex::Seq(
                Box::new(Regex::Char('a')),
                Box::new(Regex::Char('a')),
            )),
        )))));

        let r2 = Regex::Plus(Box::new(Regex::Plus(Box::new(Regex::Seq(
            Box::new(Regex::Between(Box::new(Regex::Char('a')), 19, 19)),
            Box::new(Regex::Optional(Box::new(Regex::Char('a')))),
        )))));
        // 120 a's
        assert!(matcher(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            r1.clone(),
        )); // True
        // 131 a's
        assert!(!matcher(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            r1.clone(),
        )); // False
        assert!(!matcher(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            r1,
        )); // False
        assert!(matcher(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            r2.clone(),
        )); // True
        assert!(!matcher(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            r2.clone(),
        )); // False
        assert!(matcher(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            r2,
        )); // True
    }

}

