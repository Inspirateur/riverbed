use std::{collections::{BTreeMap, BTreeSet}, str::FromStr};

use nom::{
    bytes::complete::tag, character::complete::{alpha1, line_ending, multispace0, multispace1, space0, space1}, combinator::{eof, fail, opt}, error::{Error, ParseError}, multi::{many1, separated_list0, separated_list1}, sequence::delimited, IResult, Input, Parser
};
use ron::de::SpannedError;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BlockFrag {
    SetName(String),
    Ident(String)
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum BlockFlag {
    Renewable(u32),
    Transparent,
    Furnace(u32)
}

impl FromStr for BlockFlag {
    type Err = SpannedError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ron::from_str(&s)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct AddBlock(pub(crate) (Vec<BlockFrag>, BTreeSet<BlockFlag>));

#[derive(Debug)]
struct BlockSet {
    name: String,
    variants: BTreeSet<String>,
}

#[derive(Debug)]
pub(crate) struct IR {
    pub(crate) sets: BTreeMap<String, BTreeSet<String>>,
    pub(crate) decl: Vec<AddBlock>,
}

#[derive(Debug)]
enum Either<Left, Right> {
    Left(Left),
    Right(Right),
}

pub fn parse_file(input: &str) -> IResult<&str, IR> {
    let (input, res): (&str, _) = many1(ws(parse_statement)).parse(input)?;
    let (input, _) = eof(input)?;
    let mut sets = BTreeMap::new();
    let mut decl = Vec::new();
    for either in res {
        match either {
            Either::Left(block_set) => { sets.insert(block_set.name, block_set.variants); }
            Either::Right(add_block) => { decl.push(add_block); }
        }
    }
    Ok((input, IR { sets, decl }))
}

fn statement_end(input: &str) -> IResult<&str, ()> {
    line_ending(input).map(|(input, _)| (input, ())).or_else(|_: nom::Err<Error<&str>>| eof(input).map(|(input, _)| (input, ())))
}

fn parse_statement(input: &str) -> IResult<&str, Either<BlockSet, AddBlock>> {
    let (input, stmt) = parse_set(input)
        .map(|(input, set)| (input, Either::Left(set)))
        .or_else(|_| parse_decl(input).map(|(input, decl)| (input, Either::Right(decl))))?;
    let (input, _) = space0(input)?;
    let (input, _) = statement_end(input)?;
    Ok((input, stmt))
}

fn parse_set(input: &str) -> IResult<&str, BlockSet> {
    let (input, _) = tag("set")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, name) = parse_ident(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = tag("{")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, variants) = separated_list1(ws(tag(",")), parse_ident).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("}")(input)?;
    Ok((input, BlockSet { name: name.to_string(), variants: BTreeSet::from_iter(variants.into_iter().map(String::from)) }))
}

fn parse_decl(input: &str) -> IResult<&str, AddBlock> {
    let (input, (_, _, block_pattern, flags_opt)) = ((tag("block"), space1, many1(parse_block_frag), opt((space1, parse_block_flags)))).parse(input)?;
    let flags = match flags_opt {
        None => BTreeSet::new(),
        Some((_, flags)) => flags
    };
    Ok((input, AddBlock((block_pattern, flags))))
}

fn parse_block_frag(input: &str) -> IResult<&str, BlockFrag> {
    parse_set_name(input).or(parse_block_ident(input))
}

fn parse_set_name(input: &str) -> IResult<&str, BlockFrag> {
    let (input, ident) = delimited(tag("{"), parse_ident, tag("}")).parse(input)?;
    Ok((input, BlockFrag::SetName(ident.to_string())))
}

fn parse_block_ident(input: &str) -> IResult<&str, BlockFrag> {
    let (input, ident) = parse_ident(input)?;
    Ok((input, BlockFrag::Ident(ident.to_string())))
}

fn parse_ident(input: &str) -> IResult<&str, &str> {
    alpha1(input)
}

fn parse_block_flags(input: &str) -> IResult<&str, BTreeSet<BlockFlag>> {
    let (input, flags) = separated_list0(space1, parse_block_flag).parse(input)?;
    Ok((input, BTreeSet::from_iter(flags)))
}

fn parse_block_flag(input: &str) -> IResult<&str, BlockFlag> {
    let (input_ok, flag_str) = input.split_at_position_complete(char::is_whitespace)?;
    if let Ok(flag) = BlockFlag::from_str(flag_str) {
        Ok((input_ok, flag))
    } else {
        fail().parse(input)
    }
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and 
/// trailing whitespace, returning the output of `inner`.
fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(inner: F) -> impl Parser<&'a str, Output = O, Error = E>
  where
  F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
  delimited(
    multispace0,
    inner,
    multispace0
  )
}

#[cfg(test)]
mod tests {
    use crate::parse::*;

    #[test]
    fn test_parse_set() {
        let blockdef = r#"set Wood {
            Acacia,
            Birch,
            Oak,
            Spruce
        }"#;
        let (_input, ir) = parse_set(blockdef).unwrap();
        println!("{ir:?}");
    }

    #[test]
    fn test_parse_file() {
        let blockdef = r#"
        set Wood {
            Acacia,
            Birch,
            Oak,
            Spruce
        }
        
        block Stripped{Wood}Log
        
        block IronOre renewable(10)"#;
        let (_input, ir) = parse_file(blockdef).unwrap();
        println!("{ir:?}");
    }

    #[test]
    fn test_incorrect_flag() {
        let blockdef = r#"block IronOre apodhzipa"#;
        assert!(parse_statement(&blockdef).is_err())
    }

    #[test]
    fn test_parse_flag() {
        let blockdef = r#"block IronOre renewable(10)"#;
        let (_, ir) = parse_decl(blockdef).unwrap();
        assert_eq!(ir, AddBlock((vec![BlockFrag::Ident("IronOre".to_string())], BTreeSet::from([BlockFlag::Renewable(10)]))));
    }
}
