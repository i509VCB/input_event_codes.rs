use std::ops::{RangeFrom, RangeTo};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::{
        self,
        complete::{char, line_ending, multispace0, newline, not_line_ending, space0, space1},
        is_alphanumeric,
    },
    combinator::{map, opt, value},
    error::{ErrorKind, ParseError},
    multi::many0,
    sequence::{delimited, pair, preceded, separated_pair, tuple},
    AsChar, IResult, InputIter, Slice,
};

#[derive(Debug, PartialEq, Eq)]
pub struct Define<'a> {
    pub name: &'a str,
    pub expression: Expression<'a>,
    pub comment: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expression<'a> {
    Constant(u32),

    Expression { other: &'a str, add: Option<u32> },
}

/// See pull request: https://github.com/Geal/nom/pull/1397/files
fn hex_u32<I, E: ParseError<I>>(input: I) -> IResult<I, u32, E>
where
    I: InputIter,
    I: Slice<RangeFrom<usize>> + Slice<RangeTo<usize>>,
    <I as InputIter>::Item: AsChar,
{
    // Do not parse more than 8 characters for a u32
    let mut chars = input
        .iter_elements()
        .take(8)
        // Can replace map-take_while-flatten with `.map_while` once stabilized
        .map(|c| c.as_char().to_digit(16))
        .take_while(|digit| digit.is_some())
        .flatten();
    if let Some(first) = chars.next() {
        let (num, len) = chars.fold((first, 1), |(acc, len), item| (16 * acc + item, len + 1));
        Ok((input.slice(len..), num))
    } else {
        Err(nom::Err::Error(E::from_error_kind(
            input,
            ErrorKind::HexDigit,
        )))
    }
}

/// Parse a comment, returning the content and the remaining input.
///
/// Note that this function parses the content between `/*` and the first `*/` including whitespace.
/// For example, a comment `/* foo */` would have a content of `" foo "`.
fn parse_comment(input: &str) -> IResult<&str, &str> {
    delimited(tag("/*"), take_until("*/"), tag("*/"))(input)
}

/// Consume an `#endif` statement.
fn consume_endif(input: &str) -> IResult<&str, ()> {
    value((), tag("#endif"))(input)
}

fn parse_define_expression(input: &str) -> IResult<&str, Expression> {
    map(
        delimited(
            char('('),
            separated_pair(
                parse_define_deferred,
                // space0 because a + sign may have space around it
                delimited(space0, char('+'), space0),
                character::complete::u32,
            ),
            char(')'),
        ),
        |(mut expression, add_parsed)| {
            match expression {
                Expression::Expression { ref mut add, .. } => {
                    *add = Some(add_parsed);
                }

                _ => unreachable!(),
            }

            expression
        },
    )(input)
}

fn parse_define_constant_u32(input: &str) -> IResult<&str, Expression> {
    map(character::complete::u32, Expression::Constant)(input)
}

fn parse_define_constant_hex_u32(input: &str) -> IResult<&str, Expression> {
    map(preceded(tag("0x"), hex_u32), Expression::Constant)(input)
}

fn parse_define_deferred(input: &str) -> IResult<&str, Expression> {
    map(
        take_while(|c: char| c.is_alphanumeric() || c == '_'),
        |other| Expression::Expression { other, add: None },
    )(input)
}

fn parse_define_value(input: &str) -> IResult<&str, Expression> {
    alt((
        // Try to parse with hexadecimal first
        parse_define_constant_hex_u32,
        // Try to parse a define that has a constant value
        parse_define_constant_u32,
        // An expression
        parse_define_expression,
        // Name of another define (deferred)
        parse_define_deferred,
    ))(input)
}

/// Parse an `#define`, returning an expression representing the value of the define.
///
/// This will not read the name of the define of the
fn parse_define(input: &str) -> IResult<&str, Define> {
    let define_name = delimited(
        // Consume the #define
        pair(tag("#define"), space1),
        // Parse the name of the define
        take_while(|c: char| is_alphanumeric(c as u8) || c == '_'),
        // Consume the spaces on the same
        space1,
    );

    map(
        tuple((
            define_name,
            parse_define_value,
            opt(preceded(space1, parse_comment)),
        )),
        |(name, expression, comment)| Define {
            name,
            expression,
            comment,
        },
    )(input)
}

/// Consume an `#ifndef` statement,
fn consume_ifndef(input: &str) -> IResult<&str, ()> {
    value(
        (),
        tuple((
            tag("#ifndef"),
            // There must be a space separating the pre-processor declaration and the expression.
            space1,
            // Advance until we reach a line ending to skip over the define name
            not_line_ending,
        )),
    )(input)
}

fn consume_ws_and_comments(input: &str) -> IResult<&str, ()> {
    value(
        (),
        many0(alt((
            value((), newline),
            value((), pair(parse_comment, opt(multispace0))),
        ))),
    )(input)
}

fn parse_defines(input: &str) -> IResult<&str, Vec<Define>> {
    many0(preceded(opt(consume_ws_and_comments), parse_define))(input)
}

fn consume_define_no_value(input: &str) -> IResult<&str, ()> {
    value((), delimited(tag("#define"), not_line_ending, line_ending))(input)
}

pub fn parse_file(input: &str) -> IResult<&str, Vec<Define>> {
    delimited(
        tuple((
            opt(consume_ws_and_comments),
            consume_ifndef,
            newline,
            // Consume the #define _UAPI_INPUT_EVENT_CODES_H
            consume_define_no_value,
        )),
        parse_defines,
        preceded(opt(consume_ws_and_comments), consume_endif),
    )(input)
}

#[cfg(test)]
mod test {
    use crate::parse::{
        consume_define_no_value, consume_ifndef, consume_ws_and_comments, parse_defines, Define,
        Expression,
    };

    use super::parse_define;

    #[test]
    fn consume_header() {
        const HEADER: &str = r#"/* SPDX-License-Identifier: GPL-2.0-only WITH Linux-syscall-note */
        /*
         * Input event codes
         *
         *    *** IMPORTANT ***
         * This file is not only included from C-code but also from devicetree source
         * files. As such this file MUST only contain comments and defines.
         *
         * Copyright (c) 1999-2002 Vojtech Pavlik
         * Copyright (c) 2015 Hans de Goede <hdegoede@redhat.com>
         *
         * This program is free software; you can redistribute it and/or modify it
         * under the terms of the GNU General Public License version 2 as published by
         * the Free Software Foundation.
         */
"#;

        assert_eq!(consume_ws_and_comments(HEADER), Ok(("", ())))
    }

    #[test]
    fn test_define_no_comment() {
        assert_eq!(
            consume_define_no_value("#define _UAPI_INPUT_EVENT_CODES_H\n"),
            Ok(("", ()))
        );
    }

    #[test]
    fn test_ifndef() {
        assert_eq!(consume_ifndef("#ifndef _TEST"), Ok(("", ())))
    }

    #[test]
    fn parse_lit_number() {
        assert_eq!(
            parse_define("#define SYN_REPORT		0"),
            Ok((
                "",
                Define {
                    name: "SYN_REPORT",
                    expression: Expression::Constant(0),
                    comment: None
                }
            ))
        );
    }

    #[test]
    fn parse_lit_hex() {
        assert_eq!(
            parse_define("#define EV_MAX			0x1f"),
            Ok((
                "",
                Define {
                    name: "EV_MAX",
                    expression: Expression::Constant(0x1F),
                    comment: None
                }
            ))
        );
    }

    #[test]
    fn parse_lit_hex_with_comment() {
        assert_eq!(
            parse_define("#define ABS_MT_SLOT		0x2f	/* MT slot being modified */"),
            Ok((
                "",
                Define {
                    name: "ABS_MT_SLOT",
                    expression: Expression::Constant(0x2F),
                    comment: Some(" MT slot being modified ")
                }
            ))
        );
    }

    #[test]
    fn parse_define_deferred() {
        assert_eq!(
            parse_define("#define KEY_MIN_INTERESTING	KEY_MUTE"),
            Ok((
                "",
                Define {
                    name: "KEY_MIN_INTERESTING",
                    expression: Expression::Expression {
                        other: "KEY_MUTE",
                        add: None
                    },
                    comment: None
                }
            ))
        );
    }

    #[test]
    fn parse_define_expression() {
        assert_eq!(
            parse_define("#define KEY_CNT			(KEY_MAX+1)"),
            Ok((
                "",
                Define {
                    name: "KEY_CNT",
                    expression: Expression::Expression {
                        other: "KEY_MAX",
                        add: Some(1)
                    },
                    comment: None
                }
            ))
        );
    }

    #[test]
    fn parse_define_expression_ws() {
        assert_eq!(
            parse_define("#define INPUT_PROP_CNT			(INPUT_PROP_MAX + 1)"),
            Ok((
                "",
                Define {
                    name: "INPUT_PROP_CNT",
                    expression: Expression::Expression {
                        other: "INPUT_PROP_MAX",
                        add: Some(1)
                    },
                    comment: None
                }
            ))
        )
    }

    #[test]
    fn parse_multiline_comment_define() {
        assert_eq!(
            parse_define("#define SW_RFKILL_ALL		0x03  /* rfkill master switch, type \"any\"\n        set = radio enabled */"),
            Ok((
                "",
                Define {
                    name: "SW_RFKILL_ALL",
                    expression: Expression::Constant(0x03),
                    comment: Some(" rfkill master switch, type \"any\"\n        set = radio enabled ")
                }
            ))
        )
    }

    #[test]
    fn parse_define_no_comment_attached() {
        assert_eq!(
            parse_define("#define SW_MICROPHONE_INSERT	0x04\n/* set = inserted */"),
            Ok((
                "\n/* set = inserted */",
                Define {
                    name: "SW_MICROPHONE_INSERT",
                    expression: Expression::Constant(0x04),
                    comment: None
                }
            ))
        )
    }

    #[test]
    fn parse_multiple() {
        assert_eq!(
            parse_defines("#define EV_MAX			0x1f\n#define EV_MAX			0x1f"),
            Ok((
                "",
                vec![
                    Define {
                        name: "EV_MAX",
                        expression: Expression::Constant(0x1F),
                        comment: None
                    },
                    Define {
                        name: "EV_MAX",
                        expression: Expression::Constant(0x1F),
                        comment: None
                    }
                ]
            ))
        );
    }

    #[test]
    fn parse_multiple_ws() {
        assert_eq!(
            parse_defines("#define EV_MAX			0x1f\n\n#define EV_MAX			0x1f"),
            Ok((
                "",
                vec![
                    Define {
                        name: "EV_MAX",
                        expression: Expression::Constant(0x1F),
                        comment: None
                    },
                    Define {
                        name: "EV_MAX",
                        expression: Expression::Constant(0x1F),
                        comment: None
                    }
                ]
            ))
        );
    }

    #[test]
    fn parse_multiple_ws_with_comment() {
        assert_eq!(
            parse_defines("#define EV_MAX			0x1f\n/* a comment */\n#define EV_MAX			0x1f"),
            Ok((
                "",
                vec![
                    Define {
                        name: "EV_MAX",
                        expression: Expression::Constant(0x1F),
                        comment: None
                    },
                    Define {
                        name: "EV_MAX",
                        expression: Expression::Constant(0x1F),
                        comment: None
                    }
                ]
            ))
        );
    }
}
