use std::borrow::Cow;

use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag, take_while_m_n},
    character::complete::{anychar, i32, i64, none_of, one_of},
    combinator::{all_consuming, map_res, opt, success, verify},
    error::Error,
    multi::{many0, many1, separated_list0},
    sequence::tuple,
    IResult,
};

use crate::{Expression, List, Part, Sequence};

// Parse a plain string
fn plain_str(escape_chars: &str) -> impl FnMut(&str) -> IResult<&str, Cow<str>> + '_ {
    move |input: &str| -> IResult<&str, Cow<str>> {
        let (input, string) = escaped(is_not(escape_chars), '\\', one_of(escape_chars))(input)?;
        if string.contains("\\") {
            let mut built = String::with_capacity(string.len());
            let mut iter = string.chars();
            while let Some(next) = iter.next() {
                built.push(if next == '\\' {
                    // Nom should make sure no trailing backslashes were present.
                    iter.next().unwrap()
                } else {
                    next
                });
            }

            Ok((input, std::borrow::Cow::Owned(built)))
        } else {
            Ok((input, std::borrow::Cow::Borrowed(string)))
        }
    }
}

/// A top-level plain match, which may not be empty and may contain unescaped
/// commas.
fn top_plain(input: &str) -> IResult<&str, Part<'_>> {
    let (input, s) = verify(plain_str("\\{}"), |s: &str| !s.is_empty())(input)?;
    Ok((input, Part::Plain(s)))
}

/// A non-top-level plain match, which may not be empty and may not contain
/// unescaped commas.
fn list_plain(input: &str) -> IResult<&str, Part<'_>> {
    let (input, s) = verify(plain_str("\\{},"), |s: &str| !s.is_empty())(input)?;
    Ok((input, Part::Plain(s)))
}

/// Always succeeds with an empty plain.
fn empty_plain(input: &str) -> IResult<&str, Part<'_>> {
    success(Part::Plain(Cow::Borrowed("")))(input)
}

fn sequence_char(input: &str) -> IResult<&str, char> {
    let (input, c) = none_of(".{},")(input)?;
    if c == '\\' {
        anychar(input)
    } else {
        Ok((input, c))
    }
}

fn number_sequence_incr(input: &str) -> IResult<&str, u64> {
    let (input, _) = tag("..")(input)?;
    let (input, incr) = i64(input)?;
    let incr = incr.abs() as u64;
    let incr = if incr < 1 { 1 } else { incr };
    Ok((input, incr))
}
fn char_sequence_incr(input: &str) -> IResult<&str, u32> {
    let (input, _) = tag("..")(input)?;
    let (input, incr) = i32(input)?;
    let incr = incr.abs() as u32;
    let incr = if incr < 1 { 1 } else { incr };
    Ok((input, incr))
}

fn number_sequence(input: &str) -> IResult<&str, Part<'_>> {
    let (input, _) = tag("{")(input)?;
    let (input, start) = i64(input)?;
    let (input, _) = tag("..")(input)?;
    let (input, end) = i64(input)?;
    let (input, incr) = opt(number_sequence_incr)(input)?;
    let (input, _) = tag("}")(input)?;
    Ok((
        input,
        Part::Sequence(Sequence::Int(crate::sequence::Sequence {
            start,
            end,
            incr: incr.unwrap_or(1),
        })),
    ))
}

fn char_sequence(input: &str) -> IResult<&str, Part<'_>> {
    let (input, _) = tag("{")(input)?;
    let (input, start) = sequence_char(input)?;
    let (input, _) = tag("..")(input)?;
    let (input, end) = sequence_char(input)?;
    let (input, incr) = opt(char_sequence_incr)(input)?;
    let (input, _) = tag("}")(input)?;
    Ok((
        input,
        Part::Sequence(Sequence::Char(crate::sequence::Sequence {
            start,
            end,
            incr: incr.unwrap_or(1),
        })),
    ))
}

fn sequence(input: &str) -> IResult<&str, Part<'_>> {
    alt((number_sequence, char_sequence))(input)
}

fn list_expression(input: &str) -> IResult<&str, Part<'_>> {
    let (input, parts) = many1(alt((sequence, list, list_plain)))(input)?;
    Ok((input, Part::Expression(Expression(parts))))
}

fn list(input: &str) -> IResult<&str, Part<'_>> {
    let (input, _) = tag("{")(input)?;
    let (input, items) = separated_list0(tag(","), alt((list_expression, empty_plain)))(input)?;
    let (input, _) = tag("}")(input)?;
    Ok((input, Part::List(List(items))))
}

pub fn expression(input: &str) -> IResult<&str, Expression<'_>> {
    let (input, parts) = all_consuming(many0(alt((sequence, list, top_plain))))(input)?;
    Ok((input, Expression(parts)))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_simple_list() {
        let expression = expression("{a,b,c}").unwrap().1;
        let generated: Result<Vec<_>, _> = expression.into_iter().collect();
        let expected: Vec<_> = vec!["a", "b", "c"];
        assert_eq!(generated.unwrap(), expected);
    }
    #[test]
    fn test_list_with_empty_part() {
        let expression = expression("{a,,c}").unwrap().1;
        let generated: Result<Vec<_>, _> = expression.into_iter().collect();
        let expected: Vec<_> = vec!["a", "", "c"];
        assert_eq!(generated.unwrap(), expected);
    }
}
