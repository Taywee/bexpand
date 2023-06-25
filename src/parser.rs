use std::borrow::Cow;

use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag},
    character::complete::{anychar, none_of, one_of, u32, u64},
    combinator::{all_consuming, complete, opt, success, verify},
    error::ParseError,
    multi::{many0, many1, separated_list0},
    IResult,
};

use crate::{Expression, List, Part, Sequence};

// Parse a plain string
fn plain_str<'a, E: ParseError<&'a str>>(
    escape_chars: &'static str,
) -> impl FnMut(&'a str) -> IResult<&'a str, Cow<'a, str>, E> + '_ {
    move |input: &'a str| -> IResult<&'a str, Cow<'a, str>, E> {
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
fn top_plain<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Part<'_>, E> {
    let (input, s) = verify(plain_str("\\{}"), |s: &str| !s.is_empty())(input)?;
    Ok((input, Part::Plain(s)))
}

/// A non-top-level plain match, which may not be empty and may not contain
/// unescaped commas.
fn list_plain<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Part<'_>, E> {
    let (input, s) = verify(plain_str("\\{},"), |s: &str| !s.is_empty())(input)?;
    Ok((input, Part::Plain(s)))
}

/// Always succeeds with an empty plain.
fn empty_plain<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Part<'_>, E> {
    success(Part::Plain(Cow::Borrowed("")))(input)
}

fn sequence_char<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, char, E> {
    let (input, c) = none_of(".{},")(input)?;
    if c == '\\' {
        anychar(input)
    } else {
        Ok((input, c))
    }
}

fn number_sequence_incr<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, u64, E> {
    let (input, _) = tag("..")(input)?;
    let (input, incr) = u64(input)?;
    let incr = if incr < 1 { 1 } else { incr };
    Ok((input, incr))
}
fn char_sequence_incr<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, u32, E> {
    let (input, _) = tag("..")(input)?;
    let (input, incr) = u32(input)?;
    let incr = if incr < 1 { 1 } else { incr };
    Ok((input, incr))
}

fn number_sequence<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Part<'_>, E> {
    let (input, _) = tag("{")(input)?;
    let (input, equal) = opt(tag("="))(input)?;
    let pre_start_len = input.len();
    let (input, start) = i64(input)?;
    let post_start_len = input.len();
    let (input, _) = tag("..")(input)?;
    let pre_end_len = input.len();
    let (input, end) = i64(input)?;
    let post_end_len = input.len();
    let (input, incr) = opt(number_sequence_incr)(input)?;
    let (input, _) = tag("}")(input)?;
    Ok((
        input,
        Part::Sequence(Sequence::Int {
            width: equal.map(|_| {
                let start_width = pre_start_len - post_start_len;
                let end_width = pre_end_len - post_end_len;
                start_width.max(end_width)
            }),
            sequence: crate::sequence::Sequence {
                start,
                end,
                incr: incr.unwrap_or(1),
            },
        }),
    ))
}

fn char_sequence<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Part<'_>, E> {
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

fn sequence<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Part<'_>, E> {
    alt((number_sequence, char_sequence))(input)
}

fn list_expression<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Part<'_>, E> {
    // A list expression may not be empty and may not contain any non-empty plain parts.
    let (input, parts) = many1(alt((sequence, list, list_plain)))(input)?;
    Ok((input, Part::Expression(Expression(parts))))
}

fn list<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Part<'_>, E> {
    let (input, _) = tag("{")(input)?;
    // A list may contain empty plain parts.
    let (input, items) = separated_list0(tag(","), alt((list_expression, empty_plain)))(input)?;
    let (input, _) = tag("}")(input)?;
    Ok((input, Part::List(List(items))))
}

pub fn expression<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Expression<'_>, E> {
    // A top level expression may be empty, and may not contain any non-empty plain parts
    let (input, parts) = all_consuming(complete(many0(alt((sequence, list, top_plain)))))(input)?;
    Ok((input, Expression(parts)))
}
