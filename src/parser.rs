use std::rc::Rc;
use combine::{attempt, Stream, Parser, parser, skip_many1, many1, many, choice, between, not_followed_by};
use combine::error::{Consumed, ParseError};
use combine::char::{string, space, alpha_num, letter, digit};
use combine::parser::combinator::recognize;

use super::syntax;

static RESERVED: &'static [&'static str] = &["do", "end", "replicate"];

fn white_space<I>() -> impl Parser<Input = I>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many(space()).map(|x : Vec<char>| x.into_iter().collect::<Vec<char>>())
}

parser! {
fn lex_char[I](c : char)(I) -> char
where [I: Stream<Item = char>]
{
    combine::parser::char::char(*c).skip(white_space())
}
}

fn float<I>() -> impl Parser<Input = I, Output = f64>
where
    I: Stream<Item = char>,
    I: combine::RangeStreamOnce,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    <I as combine::StreamOnce>::Range: combine::stream::Range,
    <I as combine::StreamOnce>::Range : std::string::ToString,
{
    recognize((skip_many1(digit()), lex_char('.'), skip_many1(digit())))
        .skip(white_space())
        .map(|f : Vec<char>| {
             let s : String = f.into_iter().collect();
             let t : &str = s.as_str();
             let u = t.parse::<f64>();
             u.unwrap()
         })
}

fn integer<I>() -> impl Parser<Input = I, Output = usize>
where
    I: Stream<Item = char>,
    I: combine::RangeStreamOnce,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    <I as combine::StreamOnce>::Range: combine::stream::Range,
    <I as combine::StreamOnce>::Range : std::string::ToString,
{
    recognize(skip_many1(digit()).skip(not_followed_by(lex_char('.'))))
        .skip(white_space())
        .map(|f : Vec<char>| {
             let s : String = f.into_iter().collect();
             let t : &str = s.as_str();
             let u = t.parse::<usize>();
             u.unwrap()
         })
}

parser! {
fn reserved[I]()(I) -> &'static str
where [I: Stream<Item = char>]
{
    choice((
        attempt(keyword("do")),
        attempt(keyword("end")),
        attempt(keyword("replicate"))
    ))
}
}

parser! {
fn ip[I]()(I) -> String
where [I: Stream<Item = char>]
{
    fn f(s1 : char, mut s2 : String) -> String {
        s2.insert(0, s1);
        s2
    }
    letter().and(many(alpha_num())).skip(white_space())
        .map(|(s1, s2) : (char, String)| f(s1, s2))
}
}

parser! {
fn identifier[I]()(I) -> String
where [I: Stream<Item = char>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      I: combine::RangeStreamOnce]
{
    combine::parser::function::parser(|input : &mut I| {
        let position = input.position();
        match ip().parse_stream(input) {
            Ok ((s, consumed)) => match RESERVED.iter().find(|t| s == **t) {
                Some (_) => { // this is a terrible error message
                    Err (Consumed::Empty (I::Error::empty(position).into()))
                },
                None => Ok ((s, consumed))
            },
            Err (e) => Err (e)
        }
    })
}
}

parser! {
fn keyword[I](s : &'static str)(I) -> &'static str
where [I: Stream<Item = char>]
{
    string(s).skip(white_space())
}
}

parser! {
fn action[I]()(I) -> syntax::Act
where [I: Stream<Item = char>,
      I: combine::RangeStreamOnce]
{
    let qmark = lex_char('?');
    let expmark = lex_char('!');

    let act_rec = (qmark, identifier(), lex_char(';')).map(|a| syntax::Act::Input (a.1));
    let act_send = (expmark, identifier(), lex_char(';')).map(|a| syntax::Act::Output (a.1));
    act_rec.or(act_send)
}
}

parser! {
fn ap[I]()(I) -> (syntax::Act, syntax::Process)
where [I: Stream<Item = char>,
      <I as combine::StreamOnce>::Range: std::fmt::Display,
      I: combine::RangeStreamOnce,
      <I as combine::StreamOnce>::Range: combine::stream::Range]
{
    action().and(process())
}
}

fn process_<I>() -> impl Parser<Input = I, Output = syntax::Process>
    where I: Stream<Item = char>,
          // Necessary due to rust-lang/rust#24159
          I::Error: ParseError<I::Item, I::Range, I::Position>,
          <I as combine::StreamOnce>::Range: combine::stream::Range,
          I: combine::RangeStreamOnce,
          <I as combine::StreamOnce>::Range: std::fmt::Display
{
    fn prepend<T>(elem : T, mut v : Vec<T>) -> Vec<T> {
        v.insert(0, elem);
        v
    }
    let parallel = between(lex_char('('), lex_char(')'), process().and(many1(lex_char('|').with(process().map(|p| Rc::new(p))))))
        .map(|(p1, plist)| syntax::Process::Parallel (Rc::new(prepend(Rc::new(p1), plist))));
    let actionproc = ap().map(|(a, p)| syntax::Process::Action (a, Rc::new(p)));
    let choose = keyword("do")
        .with(ap().and(many1(keyword("or").with(ap()).map(|(a, p)| (a, Rc::new(p))))))
        .map(|((a, p1), plist)| syntax::Process::Choice (Rc::new(prepend((a, Rc::new(p1)), plist))));
    let inst = identifier()
        .skip((lex_char('('), lex_char(')')))
        .map(|n : String| syntax::Process::Instance (n.to_string()));
    let repeat = integer()
        .skip(keyword("of"))
        .and(process())
        .map(|(i, p)| syntax::Process::Repitition (i, Rc::new(p)));
    let rep = keyword("replicate")
        .with(action())
        // .and(between(lex_char('{'), lex_char('}'), process()))
        .and(process())
        .map(|(a, p)| syntax::Process::Replication (a, Rc::new(p)));
    let terminate = keyword("end").map(|_| syntax::Process::Termination);
    let nesteddecl = between(lex_char('('), lex_char(')'), 
        many1(declaration().map(|d| Rc::new(d))).and(process()))
        // many1(declaration().map(|d| d)).and(process()))
        .map(|(d, p) : (Vec<Rc<syntax::Declaration>>, syntax::Process)| syntax::Process::NestedDecl (Rc::new(d), Rc::new(p)));
                             
    parallel
        .or(terminate)
        .or(choose)
        .or(actionproc)
        .or(inst)
        .or(repeat)
        .or(rep)
        .or(nesteddecl)
}

parser!{
fn process[I]()(I) -> syntax::Process
where [I: Stream<Item = char>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      I: combine::RangeStreamOnce,
      <I as combine::StreamOnce>::Range: std::fmt::Display]
{
    process_()
}
}

fn declaration_<I>() -> impl Parser<Input = I, Output = syntax::Declaration>
    where I: Stream<Item = char>,
          // Necessary due to rust-lang/rust#24159
          I::Error: ParseError<I::Item, I::Range, I::Position>,
          I: combine::RangeStreamOnce,
          <I as combine::StreamOnce>::Range: combine::stream::Range,
          <I as combine::StreamOnce>::Range: std::fmt::Display,
{
    let newchan = keyword("new")
        .with(identifier())
        .skip(lex_char('@'))
        .and(float())
        .map(|(c, r)| syntax::Declaration::NewChannel (c, r));
    let runproc = keyword("run")
        .with(process())
        .map(|p| syntax::Declaration::Run (Rc::new(p)));
    let def = keyword("let")
        .with(identifier())
        .skip(lex_char('='))
        .and(process())
        .map(|(i, p)| syntax::Declaration::Def (i, Rc::new(p)));

    newchan
        .or(runproc)
        .or(def)
}

parser!{
fn declaration[I]()(I) -> syntax::Declaration
where [I: Stream<Item = char>,
      I: combine::RangeStreamOnce,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      <I as combine::StreamOnce>::Range: std::fmt::Display]
{
    declaration_()
}
}

parser! {
pub fn program[I]()(I) -> syntax::Program
where [I: Stream<Item = char>,
      I: combine::RangeStreamOnce,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      <I as combine::StreamOnce>::Range: std::fmt::Display
    ]
{
    many1(declaration().map(|d : syntax::Declaration| Rc::new(d)))
        .map(|dec : Vec<Rc<syntax::Declaration>>| {
            syntax::Program::Prog (Rc::new(dec))
        })
}
}

