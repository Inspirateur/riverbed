use std::collections::{BTreeMap, BTreeSet};

use nom::{
    bytes::complete::tag, character::complete::{alpha1, multispace0, multispace1}, combinator::eof, error::ParseError, multi::{many1, separated_list1}, sequence::{delimited, tuple}, IResult
};

#[derive(Debug)]
pub(crate) enum BlockFrag {
    SetName(String),
    Ident(String)
}

#[derive(Debug)]
pub(crate) struct AddBlock(pub(crate) Vec<BlockFrag>);

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
    let (input, res): (&str, _) = many1(ws(parse_statement))(input)?;
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

fn parse_statement(input: &str) -> IResult<&str, Either<BlockSet, AddBlock>> {
    parse_set(input)
        .map(|(input, set)| (input, Either::Left(set)))
        .or_else(|_| parse_decl(input).map(|(input, decl)| (input, Either::Right(decl))))
}

fn parse_set(input: &str) -> IResult<&str, BlockSet> {
    let (input, _) = tag("set")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, name) = parse_ident(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = tag("{")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, variants) = separated_list1(ws(tag(",")), parse_ident)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("}")(input)?;
    Ok((input, BlockSet { name: name.to_string(), variants: BTreeSet::from_iter(variants.into_iter().map(String::from)) }))
}

fn parse_decl(input: &str) -> IResult<&str, AddBlock> {
    let (input, (_, _, block_pattern)) = tuple((tag("block"), multispace1, many1(parse_block_frag)))(input)?;
    Ok((input, AddBlock(block_pattern)))
}

fn parse_block_frag(input: &str) -> IResult<&str, BlockFrag> {
    delimited(tag("{"), parse_ident, tag("}"))(input)
        .map(|(input, set)| (input, BlockFrag::SetName(set.to_string())))
        .or_else(|_| parse_ident(input).map(|(input, ident)| (input, BlockFrag::Ident(ident.to_string()))))
}

fn parse_ident(input: &str) -> IResult<&str, &str> {
    alpha1(input)
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and 
/// trailing whitespace, returning the output of `inner`.
fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
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
        let (input, IR) = parse_set(blockdef).unwrap();
        println!("{IR:?}");
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
        "#;
        let (input, IR) = parse_file(blockdef).unwrap();
        println!("{IR:?}");
    }
}
