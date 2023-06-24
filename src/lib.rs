use std::fmt::Debug;
use std::str::FromStr;
use std::{borrow::Cow, char::CharTryFromError, iter};

use itertools::{Itertools, MultiProduct};
use nom::error::{convert_error, VerboseError};

mod parser;
mod sequence;

/// {a,b,c}
#[derive(Clone, Debug)]
struct List<'a>(Vec<Part<'a>>);

impl<'a> List<'a> {
    fn into_owned(self) -> List<'static> {
        List(self.0.into_iter().map(Part::into_owned).collect())
    }
}

impl<'a> IntoIterator for List<'a> {
    type Item = Result<Cow<'a, str>, CharTryFromError>;

    type IntoIter = iter::Flatten<<Vec<Part<'a>> as IntoIterator>::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().flatten()
    }
}

#[derive(Clone, Copy, Debug)]
enum Sequence {
    Int(sequence::Sequence<i64>),
    Char(sequence::Sequence<char>),
}

#[derive(Clone, Copy, Debug)]
enum SequenceIterator {
    Int(sequence::SequenceIterator<i64>),
    Char(sequence::SequenceIterator<char>),
}

impl IntoIterator for Sequence {
    type Item = Result<String, CharTryFromError>;

    type IntoIter = SequenceIterator;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Sequence::Int(s) => SequenceIterator::Int(s.into_iter()),
            Sequence::Char(s) => SequenceIterator::Char(s.into_iter()),
        }
    }
}

impl Iterator for SequenceIterator {
    type Item = Result<String, <u32 as TryInto<char>>::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SequenceIterator::Int(i) => i.next().map(|r| Ok(r.unwrap().to_string())),
            SequenceIterator::Char(i) => i.next().map(|r| r.map(|c| c.to_string())),
        }
    }
}

/// Bash-style brace expression. Can be created using TryFrom (like
/// `"foo{bar,baz}biz".try_into()`) or via FromStr
/// (`"foo{bar,baz}biz".parse()`). TryFrom is preferred, because it will avoid
/// unnecessary allocations wherever possible, and tie to the lifetime of the
/// incoming string. FromStr will make String clones in unnecessary places.
#[derive(Clone, Debug)]
pub struct Expression<'a>(Vec<Part<'a>>);

impl<'a> Expression<'a> {
    fn into_owned(self) -> Expression<'static> {
        Expression(self.0.into_iter().map(Part::into_owned).collect())
    }
}

impl FromStr for Expression<'static> {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let expression: Expression = s.try_into()?;
        Ok(expression.into_owned())
    }
}

impl<'a> TryFrom<&'a str> for Expression<'a> {
    type Error = String;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let output = parser::expression::<VerboseError<&str>>(value);
        match output {
            Ok((_, expression)) => Ok(expression),
            Err(nom::Err::Error(e) | nom::Err::Failure(e)) => return Err(convert_error(value, e)),
            _ => panic!("Somehow got an incomplete"),
        }
    }
}

impl<'a> IntoIterator for Expression<'a> {
    type Item = Result<Cow<'a, str>, CharTryFromError>;

    type IntoIter = ExpressionIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ExpressionIterator(self.0.into_iter().multi_cartesian_product())
    }
}

#[derive(Clone, Debug)]
pub struct ExpressionIterator<'a>(MultiProduct<PartIterator<'a>>);

impl<'a> Iterator for ExpressionIterator<'a> {
    type Item = Result<Cow<'a, str>, CharTryFromError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|parts| match parts.len() {
            0 => Ok(Cow::Borrowed("")),
            1 => parts.into_iter().next().unwrap(),
            _ => {
                let parts: Result<Vec<_>, _> = parts.into_iter().collect();
                let parts = parts?;
                let mut string = String::with_capacity(parts.iter().map(|s| s.len()).sum());
                for part in parts {
                    string.push_str(&part);
                }
                Ok(Cow::Owned(string))
            }
        })
    }
}

#[derive(Clone, Debug)]
enum Part<'a> {
    Plain(Cow<'a, str>),
    List(List<'a>),
    Sequence(Sequence),
    Expression(Expression<'a>),
}

impl<'a> Part<'a> {
    fn into_owned(self) -> Part<'static> {
        match self {
            Part::Plain(part) => Part::Plain(Cow::Owned(part.into_owned())),
            Part::List(part) => Part::List(part.into_owned()),
            Part::Sequence(part) => Part::Sequence(part),
            Part::Expression(part) => Part::Expression(part.into_owned()),
        }
    }
}

#[derive(Clone, Debug)]
enum PartIterator<'a> {
    Plain(iter::Once<Cow<'a, str>>),
    List(Box<<List<'a> as IntoIterator>::IntoIter>),
    Sequence(<Sequence as IntoIterator>::IntoIter),
    Expression(<Expression<'a> as IntoIterator>::IntoIter),
}

impl<'a> IntoIterator for Part<'a> {
    type Item = Result<Cow<'a, str>, CharTryFromError>;

    type IntoIter = PartIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Part::Plain(part) => PartIterator::Plain(iter::once(part.clone())),
            Part::List(part) => PartIterator::List(Box::new(part.into_iter())),
            Part::Sequence(part) => PartIterator::Sequence(part.into_iter()),
            Part::Expression(part) => PartIterator::Expression(part.into_iter()),
        }
    }
}

impl<'a> Iterator for PartIterator<'a> {
    type Item = Result<Cow<'a, str>, CharTryFromError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            PartIterator::Plain(part) => part.next().map(|s| Ok(s)),
            PartIterator::List(part) => part.next(),
            PartIterator::Sequence(part) => part.next().map(|r| r.map(|s| Cow::Owned(s))),
            PartIterator::Expression(part) => part.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_list() {
        let expression: Expression = "{a,b,c}".try_into().unwrap();
        let generated: Result<Vec<_>, _> = expression.into_iter().collect();
        let expected: Vec<_> = vec!["a", "b", "c"];
        assert_eq!(generated.unwrap(), expected);
    }

    #[test]
    fn test_list_escapes() {
        let expression: Expression = r"{a,b\,c,d\{e,f\}\\g}".try_into().unwrap();
        let generated: Result<Vec<_>, _> = expression.into_iter().collect();
        let expected: Vec<_> = vec!["a", "b,c", "d{e", r"f}\g"];
        assert_eq!(generated.unwrap(), expected);
    }
    #[test]
    fn test_nested_list() {
        let expression: Expression = r"s{a,b{c,d{e,f}g,h{i,j{k}l,m{}n}o}p,q}r"
            .try_into()
            .unwrap();
        let generated: Result<Vec<_>, _> = expression.into_iter().collect();
        let expected: Vec<_> = vec![
            "sar",
            "sbcpr",
            "sbdegpr",
            "sbdfgpr",
            "sbhiopr",
            "sbhjklopr",
            "sbhmnopr",
            "sqr",
        ];
        assert_eq!(generated.unwrap(), expected);
    }
    #[test]
    fn test_list_with_empty_part() {
        let expression: Expression = "{a,,c}".try_into().unwrap();
        let generated: Result<Vec<_>, _> = expression.into_iter().collect();
        let expected: Vec<_> = vec!["a", "", "c"];
        assert_eq!(generated.unwrap(), expected);
    }
    #[test]
    fn test_expression() {
        let expression: Expression = "a{b,c,}d{1..2}e".try_into().unwrap();
        let generated: Result<Vec<_>, _> = expression.into_iter().collect();
        let expected = vec!["abd1e", "abd2e", "acd1e", "acd2e", "ad1e", "ad2e"];
        assert_eq!(generated.unwrap(), expected);
    }
    #[test]
    fn test_char_sequence() {
        let expression: Expression = "a{d..f}g".try_into().unwrap();
        let generated: Result<Vec<_>, _> = expression.into_iter().collect();
        let expected = vec!["adg", "aeg", "afg"];
        assert_eq!(generated.unwrap(), expected);
    }
    #[test]
    fn test_negative_number_sequence() {
        let expression: Expression = "a{-10..10..3}g".try_into().unwrap();
        let generated: Result<Vec<_>, _> = expression.into_iter().collect();
        let expected = vec!["a-10g", "a-7g", "a-4g", "a-1g", "a2g", "a5g", "a8g"];
        assert_eq!(generated.unwrap(), expected);
    }
    #[test]
    fn test_decreasing_negative_number_sequence() {
        let expression: Expression = "a{-10..10..3}g".try_into().unwrap();
        let generated: Result<Vec<_>, _> = expression.into_iter().collect();
        let expected = vec!["a-10g", "a-7g", "a-4g", "a-1g", "a2g", "a5g", "a8g"];
        assert_eq!(generated.unwrap(), expected);
    }
    #[test]
    fn test_escaped_char_sequence() {
        let expression: Expression = r"a{z..\}}b{\...\{..77}c".try_into().unwrap();
        let generated: Result<Vec<_>, _> = expression.into_iter().collect();
        let expected = vec![
            "azb.c", "azb{c", "a{b.c", "a{b{c", "a|b.c", "a|b{c", "a}b.c", "a}b{c",
        ];
        assert_eq!(generated.unwrap(), expected);
    }
}
