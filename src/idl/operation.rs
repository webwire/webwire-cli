use nom::{
    bytes::complete::tag,
    character::complete::char,
    combinator::{cut, map},
    multi::separated_list,
    sequence::{pair, preceded, separated_pair, terminated},
    IResult,
};

use crate::idl::common::{
    parse_field_separator,
    parse_identifier,
    trailing_comma,
    ws,
    ws1,
    Span,
};

#[derive(Debug, PartialEq)]
pub struct Operation {
    pub name: String,
    pub in_: Option<String>,
    pub out: Option<String>,
    pub err: Option<String>,
}

struct Parameter {
    pub name: String,
    pub type_: String,
}

fn parse_parameter(input: Span) -> IResult<Span, Parameter> {
    map(
        separated_pair(parse_identifier, preceded(ws, char(':')), parse_identifier),
        |(name, type_)| Parameter {
            name: name,
            type_: type_,
        },
    )(input)
}

fn parse_parameters(input: Span) -> IResult<Span, Vec<Parameter>> {
    preceded(
        preceded(ws, char('{')),
        cut(terminated(
            separated_list(parse_field_separator, parse_parameter),
            preceded(trailing_comma, preceded(ws, char('}'))),
        )),
    )(input)
}

pub fn parse_operation(input: Span) -> IResult<Span, Operation> {
    map(
        preceded(
            terminated(tag("operation"), ws1),
            cut(pair(parse_identifier, parse_parameters)),
        ),
        |(name, params): (String, Vec<Parameter>)| {
            let mut op = Operation {
                name: name,
                in_: None,
                out: None,
                err: None,
            };
            for param in params {
                match param.name.as_str() {
                    "in" => op.in_ = Some(param.type_),
                    "out" => op.out = Some(param.type_),
                    "err" => op.err = Some(param.type_),
                    _ => { /* FIXME return an error */ }
                }
            }
            op
        },
    )(input)
}
