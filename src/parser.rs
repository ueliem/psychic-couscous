use std::rc::Rc;
use combine::{Stream, Parser, parser, many1, between, sep_by};
use combine::error::{ParseError};
use combine_language;

use super::tokenizer;
use super::tokenizer::{Token, Keyword};
use super::syntax;
use super::values::*;
use super::lambda::*;

fn pattern_<I>() -> impl Parser<Input = I, Output = Pattern>
where I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      I: combine::RangeStreamOnce
{
    let wildcard = tokenizer::underscore().map(|_| Pattern::Wildcard);
    let name = tokenizer::ident().map(|n| Pattern::Name (n));
    let tuple = between(tokenizer::lpar(), tokenizer::rpar(), sep_by(pattern(), tokenizer::comma()))
        .map(|v : Vec<Pattern>| Pattern::Tuple (v));

    wildcard.or(name).or(tuple)
}

parser!{
fn pattern[I]()(I) -> Pattern
where [I: Stream<Item = Token>,
      I: combine::RangeStreamOnce,
      <I as combine::StreamOnce>::Range: combine::stream::Range]
{
    pattern_()
}
}

fn lambda_<I>() -> impl Parser<Input = I, Output = Lambda>
where I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      I: combine::RangeStreamOnce
{
    let int = tokenizer::integer().map(|i| Lambda::IntLiteral { i : i , t : Type::Integer });
    let flt = tokenizer::float().map(|f| Lambda::FloatLiteral { f : f, t : Type::Float });
    let bt = tokenizer::keyword(Keyword::True).map(|_| Lambda::True {t : Type::Bool } );
    let bf = tokenizer::keyword(Keyword::False).map(|_| Lambda::False {t : Type::Bool });
    let var = tokenizer::ident().map(|n| Lambda::Var { v : n, t : Type::TVar });

    let fun = tokenizer::keyword(Keyword::Fun)
        .with(tokenizer::ident())
        .skip(tokenizer::rightarrow())
        .and(expr())
        .map(|(v, e)| Lambda::Abs { x : v, e : Rc::new(e), t : Type::TVar });

    let ifexpr = tokenizer::keyword(Keyword::If)
        .with(expr())
        .skip(tokenizer::keyword(Keyword::Then))
        .and(expr())
        .skip(tokenizer::keyword(Keyword::Else))
        .and(expr())
        .map(|((e1, e2), e3)| Lambda::IfExpr { c : Rc::new(e1), e1 : Rc::new(e2), e2 : Rc::new(e3), t : Type::TVar });
    let paren = between(tokenizer::lpar(), tokenizer::rpar(), expr());

    int.or(flt).or(bt).or(bf).or(var).or(fun).or(ifexpr).or(paren)
}

parser!{
fn lambda[I]()(I) -> Lambda
where [I: Stream<Item = Token>,
      I: combine::RangeStreamOnce,
      <I as combine::StreamOnce>::Range: combine::stream::Range]
{
    lambda_()
}
}

fn op(l : Lambda, o : tokenizer::Token, r : Lambda) -> Lambda {
    Lambda::BinExpr { b : o.into(), l : Rc::new(l), r : Rc::new(r), t : Type::TVar }
}

parser!{
fn op_parser[I]()(I) -> (tokenizer::Token, combine_language::Assoc)
where [I: Stream<Item = Token>,
      I: combine::RangeStreamOnce,
      <I as combine::StreamOnce>::Range: combine::stream::Range]
{
    tokenizer::binop()
        .map(|op| {
            match op {
                tokenizer::Token::Equals => 
                    (op, combine_language::Assoc { precedence : 5, fixity : combine_language::Fixity::Left }),
                tokenizer::Token::Less => 
                    (op, combine_language::Assoc { precedence : 5, fixity : combine_language::Fixity::Left }),
                tokenizer::Token::Greater => 
                    (op, combine_language::Assoc { precedence : 5, fixity : combine_language::Fixity::Left }),
                tokenizer::Token::LEq => 
                    (op, combine_language::Assoc { precedence : 5, fixity : combine_language::Fixity::Left }),
                tokenizer::Token::GEq => 
                    (op, combine_language::Assoc { precedence : 5, fixity : combine_language::Fixity::Left }),
                tokenizer::Token::NotEqual => 
                    (op, combine_language::Assoc { precedence : 5, fixity : combine_language::Fixity::Left }),
                tokenizer::Token::Plus => 
                    (op, combine_language::Assoc { precedence : 6, fixity : combine_language::Fixity::Left }),
                tokenizer::Token::Dash => 
                    (op, combine_language::Assoc { precedence : 6, fixity : combine_language::Fixity::Left }),
                tokenizer::Token::Star => 
                    (op, combine_language::Assoc { precedence : 7, fixity : combine_language::Fixity::Left }),
                tokenizer::Token::Slash => 
                    (op, combine_language::Assoc { precedence : 7, fixity : combine_language::Fixity::Left }),
                _ => panic!()
            }
        })
}
}

parser!{
fn expr[I]()(I) -> Lambda
where [I: Stream<Item = Token>,
      I: combine::RangeStreamOnce,
      <I as combine::StreamOnce>::Range: combine::stream::Range]
{
    combine_language::expression_parser(lambda(), op_parser(), op)
}
}

parser! {
fn action[I]()(I) -> syntax::Act
where [I: Stream<Item = Token>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      I: combine::RangeStreamOnce]
{
    let act_rec = (tokenizer::qmark(), tokenizer::ident(), tokenizer::semicolon()).map(|i| syntax::Act::Input (i.1));
    let act_send = (tokenizer::exmark(), tokenizer::ident(), tokenizer::semicolon()).map(|i| syntax::Act::Output (i.1));
    act_rec.or(act_send)
}
}

parser! {
fn ap[I]()(I) -> (syntax::Act, syntax::Process)
where [I: Stream<Item = Token>,
      I: combine::RangeStreamOnce,
      <I as combine::StreamOnce>::Range: combine::stream::Range]
{
    action().and(process())
}
}

fn process_<I>() -> impl Parser<Input = I, Output = syntax::Process>
    where I: Stream<Item = Token>,
          // Necessary due to rust-lang/rust#24159
          I::Error: ParseError<I::Item, I::Range, I::Position>,
          <I as combine::StreamOnce>::Range: combine::stream::Range,
          I: combine::RangeStreamOnce
{
    fn prepend<T>(elem : T, mut v : Vec<T>) -> Vec<T> {
        v.insert(0, elem);
        v
    }
    let restrict = tokenizer::keyword(Keyword::Let)
        .skip(tokenizer::keyword(Keyword::New))
        .with(tokenizer::ident())
        .skip(tokenizer::at())
        .and(tokenizer::float())
        .skip(tokenizer::keyword(Keyword::In))
        .and(process())
        .map(|((c, r), p) : ((String, f64), syntax::Process)| syntax::Process::Restriction (c, r, Rc::new(p)));
    let val = tokenizer::keyword(Keyword::Val)
        .with(pattern())
        .skip(tokenizer::equals())
        .and(lambda())
        .skip(tokenizer::keyword(Keyword::In))
        .and(process())
        .map(|((pat, lam), p) : ((Pattern, Lambda), syntax::Process)| syntax::Process::LetVal (pat, lam, Rc::new(p)));
    let parallel = between(tokenizer::lpar(), tokenizer::rpar(), process().and(many1(tokenizer::pipe().with(process().map(|p| Rc::new(p))))))
        .map(|(p1, plist)| syntax::Process::Parallel (Rc::new(prepend(Rc::new(p1), plist))));
    let actionproc = ap().map(|(a, p)| syntax::Process::Action (a, Rc::new(p)));
    let choose = tokenizer::keyword(Keyword::Do)
        .with(ap().and(many1(tokenizer::keyword(Keyword::Or).with(ap()).map(|(a, p)| (a, Rc::new(p))))))
        .map(|((a, p1), plist)| syntax::Process::Choice (Rc::new(prepend((a, Rc::new(p1)), plist))));
    let inst = tokenizer::ident()
        .and(between(tokenizer::lpar(), tokenizer::rpar(), sep_by(lambda(), tokenizer::comma())))
        .map(|(i, l) : (String, Vec<Lambda>)| syntax::Process::Instance (i, l));
    let repeat = tokenizer::integer()
        .skip(tokenizer::keyword(Keyword::Of))
        .and(process())
        .map(|(i, p)| syntax::Process::Repetition (i as usize, Rc::new(p)));
    let rep = tokenizer::keyword(Keyword::Replicate)
        .with(action())
        .and(process())
        .map(|(a, p)| syntax::Process::Replication (a, Rc::new(p)));
    let terminate = tokenizer::keyword(Keyword::End).map(|_| syntax::Process::Termination);

    restrict
        .or(val)
        .or(parallel)
        .or(terminate)
        .or(choose)
        .or(actionproc)
        .or(inst)
        .or(repeat)
        .or(rep)
}

parser!{
fn process[I]()(I) -> syntax::Process
where [I: Stream<Item = Token>,
      I::Error: ParseError<I::Item, I::Range, I::Position>,
      <I as combine::StreamOnce>::Range: combine::stream::Range,
      I: combine::RangeStreamOnce]
{
    process_()
}
}

fn declaration_<I>() -> impl Parser<Input = I, Output = syntax::Declaration>
    where I: Stream<Item = Token>,
          // Necessary due to rust-lang/rust#24159
          I::Error: ParseError<I::Item, I::Range, I::Position>,
          I: combine::RangeStreamOnce,
          <I as combine::StreamOnce>::Range: combine::stream::Range,
{
    let newchan = tokenizer::keyword(Keyword::New)
        .with(tokenizer::ident())
        .skip(tokenizer::at())
        .and(tokenizer::float())
        .map(|(c, r)| syntax::Declaration::NewChannel (c, r));
    let runproc = tokenizer::keyword(Keyword::Run)
        .with(process())
        .map(|p| syntax::Declaration::Run (Rc::new(p)));
    let val = tokenizer::keyword(Keyword::Val)
        .with(pattern())
        .skip(tokenizer::equals())
        .and(lambda())
        .map(|(p, v)| syntax::Declaration::Val (p, v));
    let def = tokenizer::keyword(Keyword::Let)
        .with(tokenizer::ident())
        .and(between(tokenizer::lpar(), tokenizer::rpar(), sep_by(pattern(), tokenizer::comma())))
        .skip(tokenizer::equals())
        .and(process())
        .map(|((c, pat), p) : ((String, Vec<Pattern>), syntax::Process)| syntax::Declaration::Def (c, pat, Rc::new(p)));

    newchan
        .or(runproc)
        .or(val)
        .or(def)
}

parser!{
fn declaration[I]()(I) -> syntax::Declaration
where [I: Stream<Item = Token>,
      I: combine::RangeStreamOnce,
      <I as combine::StreamOnce>::Range: combine::stream::Range]
{
    declaration_()
}
}

parser! {
pub fn program[I]()(I) -> syntax::Program
where [I: Stream<Item = Token>,
      I: combine::RangeStreamOnce,
      <I as combine::StreamOnce>::Range: combine::stream::Range]
{
    many1(declaration().map(|d : syntax::Declaration| Rc::new(d)))
        .map(|dec : Vec<Rc<syntax::Declaration>>| {
            syntax::Program::Prog (Rc::new(dec))
        })
}
}

