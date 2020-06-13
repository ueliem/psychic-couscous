use std::convert::TryFrom;
use combine::{optional, Stream, Parser, parser, many1, many};
use combine::error::{ParseError};
use combine::char::{space, alpha_num, letter, digit};
use combine::parser::item::{satisfy, satisfy_map};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Keyword {
    True,
    False,
    If,
    Then,
    Else,
    Fun,
    New,
    Let,
    Val,
    In,
    And,
    Do,
    Or,
    Of,
    Replicate,
    Run,
    End
}

impl TryFrom<&str> for Keyword {
    type Error = ();
    fn try_from(s : &str) -> Result<Keyword, ()> {
        match s {
            "true" => Ok (Keyword::True),
            "false" => Ok (Keyword::False),
            "if" => Ok (Keyword::If),
            "then" => Ok (Keyword::If),
            "else" => Ok (Keyword::Else),
            "fun" => Ok (Keyword::Fun),
            "new" => Ok (Keyword::New),
            "let" => Ok (Keyword::Let),
            "val" => Ok (Keyword::Val),
            "in" => Ok (Keyword::In),
            "and" => Ok (Keyword::And),
            "do" => Ok (Keyword::Do),
            "or" => Ok (Keyword::Or),
            "of" => Ok (Keyword::Of),
            "replicate" => Ok(Keyword::Replicate),
            "run" => Ok(Keyword::Run),
            "end" => Ok (Keyword::End),
            _ => Err (())
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Token {
    QMark,
    ExMark,
    At,
    Equals,
    Less,
    Greater,
    LEq,
    GEq,
    NotEqual,
    LPar,
    RPar,
    Colon,
    Semicolon,
    Pipe,
    Underscore,
    Comma,
    Plus,
    Dash,
    Star,
    Slash,
    RightArrow,
    Integer (i64),
    Float (f64),
    Keyword (Keyword),
    Identifier (String)
}

fn white_space<I>() -> impl Parser<Input = I>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many(space()).map(|x : Vec<char>| x.into_iter().collect::<Vec<char>>())
}

parser! {
fn symbol[I]()(I) -> Token
where [I: Stream<Item = char>]
{
    let symbols = vec!['?', '!', '@', '=', ';', '|', '_', ',', '+', '-', '*', '/'];
    many1(combine::parser::item::one_of(symbols)).skip(white_space()).map(|c : String| {
        match c.as_str() {
            "?" => Token::QMark,
            "!" => Token::ExMark,
            "@" => Token::At,
            "=" => Token::Equals,
            "<" => Token::Less,
            ">" => Token::Greater,
            "<=" => Token::LEq,
            ">=" => Token::GEq,
            "<>" => Token::NotEqual,
            ":" => Token::Semicolon,
            ";" => Token::Semicolon,
            "|" => Token::Pipe,
            "_" => Token::Underscore,
            "," => Token::Comma,
            "+" => Token::Plus,
            "-" => Token::Dash,
            "*" => Token::Star,
            "/" => Token::Slash,
            "=>" => Token::RightArrow,
            _ => panic!("unexpected symbol {}", c)
        }
    })
}
}

parser! {
fn lex_char[I](c : char)(I) -> char
where [I: Stream<Item = char>]
{
    combine::parser::char::char(*c).skip(white_space())
}
}

parser! {
fn leftparen[I]()(I) -> Token
where [I: Stream<Item = char>]
{
    lex_char('(').map(|_| Token::LPar)
}
}

parser! {
fn rightparen[I]()(I) -> Token
where [I: Stream<Item = char>]
{
    lex_char(')').map(|_| Token::RPar)
}
}

parser! {
fn number[I]()(I) -> Token
where [I: Stream<Item = char>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      <I as combine::StreamOnce>::Range: std::fmt::Display,
      I: combine::RangeStreamOnce]
{
    (many1(digit()),
    optional((lex_char('.'), many1(digit()))))
        .skip(white_space())
        .map(|(i, f) : (String, Option<(char, String)>)| {
            match f {
                Some ((_, s)) => {
                    let mut ii = i.clone();
                    ii.push_str(&s);
                    Token::Float (ii.parse::<f64>().unwrap())
                },
                None => Token::Integer (i.parse::<i64>().unwrap())
            }
        })
}
}

parser! {
fn ip[I]()(I) -> String
where [I: Stream<Item = char>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      <I as combine::StreamOnce>::Range: std::fmt::Display,
      I: combine::RangeStreamOnce]
{
    (many1(letter()), many(alpha_num()))
        .skip(white_space())
        .map(|s : (String, String)| {
            let mut t = s.0.clone();
            t.push_str(&s.1);
            t
        })
}
}

parser! {
fn identifier[I]()(I) -> Token
where [I: Stream<Item = char>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      <I as combine::StreamOnce>::Range: std::fmt::Display,
      I: combine::RangeStreamOnce]
{
    combine::parser::function::parser(|input : &mut I| {
        match ip().parse_stream(input) {
            Ok ((s, consumed)) => 
                match Keyword::try_from(s.as_str()) {
                    Ok (k) => {
                        Ok ((Token::Keyword (k), consumed))
                    },
                    Err (()) => Ok ((Token::Identifier (s), consumed))
                },
            Err (e) => Err (e)
        }
    })
}
}

parser! {
pub fn tok[I]()(I) -> Token // syntax::Act
where [I: Stream<Item = char>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      <I as combine::StreamOnce>::Range: std::fmt::Display,
      I: combine::RangeStreamOnce]
{
    number()
    .or(identifier())
    .or(symbol())
    .or(leftparen())
    .or(rightparen())
    .skip(white_space())
}
}

parser! {
pub fn tokenize[I]()(I) -> Vec<Token>
where [I: Stream<Item = char>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      <I as combine::StreamOnce>::Range: std::fmt::Display,
      I: combine::RangeStreamOnce]
{
    many(tok())
}
}


parser! {
pub fn qmark[I]()(I) -> Token
where [I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      I: combine::StreamOnce]
{
    satisfy(|t : Token| { match t { Token::QMark=> true, _ => false } })
}
}

parser! {
pub fn exmark[I]()(I) -> Token // syntax::Act
where [I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      I: combine::StreamOnce]
{
    satisfy(|t : Token| { match t { Token::ExMark => true, _ => false } })
}
}

parser! {
pub fn at[I]()(I) -> Token // syntax::Act
where [I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      I: combine::StreamOnce]
{
    satisfy(|t : Token| { match t { Token::At => true, _ => false } })
}
}

parser! {
pub fn equals[I]()(I) -> Token // syntax::Act
where [I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      I: combine::StreamOnce]
{
    satisfy(|t : Token| { match t { Token::Equals => true, _ => false } })
}
}

parser! {
pub fn lpar[I]()(I) -> Token // syntax::Act
where [I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      I: combine::StreamOnce]
{
    satisfy(|t : Token| { match t { Token::LPar => true, _ => false } })
}
}

parser! {
pub fn rpar[I]()(I) -> Token
where [I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      I: combine::StreamOnce]
{
    satisfy(|t : Token| { match t { Token::RPar => true, _ => false } })
}
}

parser! {
pub fn colon[I]()(I) -> Token
where [I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      I: combine::StreamOnce]
{
    satisfy(|t : Token| { match t { Token::Colon => true, _ => false } })
}
}

parser! {
pub fn semicolon[I]()(I) -> Token
where [I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      I: combine::StreamOnce]
{
    satisfy(|t : Token| { match t { Token::Semicolon => true, _ => false } })
}
}

parser! {
pub fn pipe[I]()(I) -> Token
where [I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      I: combine::StreamOnce]
{
    satisfy(|t : Token| { match t { Token::Pipe => true, _ => false } })
}
}

parser! {
pub fn underscore[I]()(I) -> Token
where [I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      I: combine::StreamOnce]
{
    satisfy(|t : Token| { match t { Token::Underscore => true, _ => false } })
}
}

parser! {
pub fn comma[I]()(I) -> Token
where [I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      I: combine::StreamOnce]
{
    satisfy(|t : Token| { match t { Token::Comma => true, _ => false } })
}
}

parser! {
pub fn rightarrow[I]()(I) -> Token
where [I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      I: combine::StreamOnce]
{
    satisfy(|t : Token| { match t { Token::RightArrow => true, _ => false } })
}
}

parser! {
pub fn binop[I]()(I) -> Token
where [I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      I: combine::StreamOnce]
{
    satisfy(|t : Token| { match t { 
        Token::Plus |
        Token::Dash |
        Token::Star |
        Token::Slash |
        Token::Equals |
        Token::Less |
        Token::Greater |
        Token::LEq |
        Token::GEq |
        Token::NotEqual => true, 
        _ => false } })
}
}

parser! {
pub fn integer[I]()(I) -> i64
where [I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      // <I as combine::StreamOnce>::Range: std::fmt::Display,
      I: combine::StreamOnce]
{
    satisfy_map(|t : Token| { match t { Token::Integer (i) => Some (i), _ => None } })
}
}

parser! {
pub fn float[I]()(I) -> f64
where [I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      // <I as combine::StreamOnce>::Range: std::fmt::Display,
      I: combine::StreamOnce]
{
    satisfy_map(|t : Token| { match t { Token::Float (f) => Some (f), _ => None } })
}
}

parser! {
pub fn keyword[I](k : Keyword)(I) -> Token
where [I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      // <I as combine::StreamOnce>::Range: std::fmt::Display,
      I: combine::StreamOnce]
{
    satisfy(|t : Token| { match (t, *k) { (Token::Keyword (tt), kk) => tt == kk, _ => false } })
}
}

parser! {
pub fn ident[I]()(I) -> String
where [I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      // <I as combine::StreamOnce>::Range: std::fmt::Display,
      I: combine::StreamOnce]
{  
    satisfy_map(|t : Token| { match t { Token::Identifier (i) => Some (i), _ => None } })
}
}

