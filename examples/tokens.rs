use nom::{
    branch::alt,
    bytes::streaming::tag,
    character::streaming::{alpha1, alphanumeric0, multispace0},
    combinator::recognize,
    sequence::tuple,
    IResult,
};
use nom_async::*;
use tokio::{
    io,
    io::{AsyncBufReadExt, AsyncWriteExt},
    stream::{StreamExt},
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Token {
    Lpar,
    Rpar,
    Sym(String),
}

pub fn token<'a>(input: &'a str) -> IResult<&'a str, Token> {
    let (i, _) = multispace0(input)?;
    alt((lpar, rpar, sym))(i)
}

pub fn lpar<'a>(input: &'a str) -> IResult<&'a str, Token> {
    let (i, _) = tag("(")(input)?;
    Ok((i, Token::Lpar))
}

pub fn rpar<'a>(input: &'a str) -> IResult<&'a str, Token> {
    let (i, _) = tag(")")(input)?;
    Ok((i, Token::Rpar))
}

pub fn sym<'a>(input: &'a str) -> IResult<&'a str, Token> {
    let (i, res) = recognize(tuple((alpha1, alphanumeric0)))(input)?;
    let res = String::from(res);
    Ok((i, Token::Sym(res)))
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut tokens = NomStream::new(io::BufReader::new(io::stdin()).lines(), token);
    let mut stdout = io::stdout();
    while let Some(tok) = tokens.next().await {
        let out = format!("tok: {:?}\n", tok);
        stdout.write(&out.as_bytes()).await?;
    }
    Ok(())
}
