use std::fmt::{Debug, Write};
use std::{borrow::Cow, char::CharTryFromError, convert::Infallible, fmt::Display, iter};

use itertools::{Itertools, MultiProduct};

mod parser;
mod sequence;

/// {a,b,c}
#[derive(Clone, Debug)]
struct List<'a>(Vec<Part<'a>>);

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

/// Cartesian product expression.
#[derive(Clone, Debug)]
pub struct Expression<'a>(Vec<Part<'a>>);

impl<'a> IntoIterator for Expression<'a> {
    type Item = Result<String, CharTryFromError>;

    type IntoIter = ExpressionIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ExpressionIterator(self.0.into_iter().multi_cartesian_product())
    }
}

#[derive(Clone, Debug)]
pub struct ExpressionIterator<'a>(MultiProduct<PartIterator<'a>>);

impl<'a> Iterator for ExpressionIterator<'a> {
    type Item = Result<String, CharTryFromError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|parts| {
            let mut string = String::new();
            for part in parts {
                write!(&mut string, "{}", part?).unwrap();
            }
            Ok(string)
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
            PartIterator::Expression(part) => part.next().map(|r| r.map(|s| Cow::Owned(s))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    #[test]
    fn test_simple_list() {
        let list = List(vec![
            Part::Plain("a".into()),
            Part::Plain("b".into()),
            Part::Plain("c".into()),
        ]);

        let values: Result<Vec<_>, _> = list.into_iter().collect();
        assert_eq!(&values.unwrap(), &["a", "b", "c"]);
    }

    #[test]
    fn test_compound_list() {
        let list = List(vec![
            Part::Plain("a".into()),
            Part::List(List(vec![Part::Plain("b".into()), Part::Plain("c".into())])),
            Part::Plain("d".into()),
        ]);

        let values: Result<Vec<_>, _> = list.into_iter().collect();
        assert_eq!(&values.unwrap(), &["a", "b", "c", "d"]);
    }

    #[test]
    fn test_compound() {
        let list = Expression(vec![
            Part::Plain("a".into()),
            Part::List(List(vec![Part::Plain("b".into()), Part::Plain("c".into())])),
            Part::Plain("d".into()),
            Part::Sequence(Sequence::Int(sequence::Sequence {
                start: 1,
                end: 2,
                incr: 1,
            })),
            Part::Plain("e".into()),
        ]);

        let values: Result<HashSet<_>, _> = list.into_iter().collect();
        let compare: HashSet<String> = [
            String::from("abd1e"),
            String::from("acd1e"),
            String::from("abd2e"),
            String::from("acd2e"),
        ]
        .into_iter()
        .collect();
        assert_eq!(values.unwrap(), compare);
    }
}
