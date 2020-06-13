use std::rc::Rc;
use std::convert::From;
use std::convert::Into;
use std::cmp::{PartialEq, PartialOrd, Ordering};

use diff_enum::common_fields;

use super::tokenizer;
use super::values::*;

#[derive(Clone, Copy, Debug)]
pub enum BinOp {
    Plus,
    Sub,
    Times,
    Div,
    Equal,
    Less,
    Greater,
    LEq,
    GEq,
    NotEqual
}

#[common_fields {
    t : Type
}]
#[derive(Clone, Debug)]
pub enum Lambda {
    IntLiteral { i : i64 },
    FloatLiteral { f : f64 },
    True,
    False,
    Var { v : String },
    Tuple { tup : Vec<Lambda> },
    Index { i : i64, e : Rc<Lambda> },
    Abs { x : String, e : Rc<Lambda> },
    App { lhs : Rc<Lambda>, rhs : Rc<Lambda> },
    IfExpr { c : Rc<Lambda>, e1 : Rc<Lambda>, e2 : Rc<Lambda> },
    BinExpr { b : BinOp, l : Rc<Lambda>, r : Rc<Lambda> }
}

impl From<tokenizer::Token> for BinOp {
    fn from(t : tokenizer::Token) -> BinOp {
        match t {
            tokenizer::Token::Plus => BinOp::Plus,
            tokenizer::Token::Dash => BinOp::Sub,
            tokenizer::Token::Star => BinOp::Times,
            tokenizer::Token::Slash => BinOp::Div,
            _ => panic!()
        }
    }
}

impl BinOp {
    pub fn eval(self, t : Type, l : Lambda, r : Lambda) -> Lambda {
        match t {
            Type::Integer | Type::Float => match self {
                    BinOp::Plus => l + r,
                    BinOp::Sub => l - r,
                    BinOp::Times => l * r,
                    BinOp::Div => l / r,
                    BinOp::Equal => Lambda::from(l == r),
                    BinOp::Less => Lambda::from(l < r),
                    BinOp::Greater => Lambda::from(l > r),
                    BinOp::LEq => Lambda::from(l <= r),
                    BinOp::GEq => Lambda::from(l >= r),
                    BinOp::NotEqual => Lambda::from(l != r),
                },
            _ => panic!()
        }
    }
}

impl Substitutable<Lambda> for Lambda {
    fn substitute (&self, src : &str, dest : Lambda) -> Lambda {
        match self {
            Lambda::IntLiteral { i : _, t : _ } => self.clone(),
            Lambda::FloatLiteral { f : _, t : _ } => self.clone(),
            Lambda::True { t : _ } => self.clone(),
            Lambda::False { t : _ } => self.clone(),
            Lambda::Var { v, t : _ } => 
                if v == src { dest.clone() } else { self.clone() },
            Lambda::Tuple { tup, t } => 
                Lambda::Tuple { tup : tup.iter().map(|x| x.substitute(src, dest.clone())).collect(), t : t.clone() },
            Lambda::Index { i, e, t } => 
                Lambda::Index { i : *i, e : Rc::new(e.substitute(src, dest.clone())), t : t.clone() },
            Lambda::Abs { x, e, t } => 
                Lambda::Abs { x : x.clone(), e : Rc::new(e.substitute(src, dest.clone())), t : t.clone() },
            Lambda::App { lhs, rhs, t } => 
                Lambda::App { lhs : Rc::new(lhs.substitute(src, dest.clone())), rhs : Rc::new(rhs.substitute(src, dest.clone())), t : t.clone() },
            Lambda::IfExpr { c, e1, e2, t } =>
                Lambda::IfExpr { c : Rc::new(c.substitute(src, dest.clone())), 
                    e1 : Rc::new(e1.substitute(src, dest.clone())), 
                    e2 : Rc::new(e2.substitute(src, dest.clone())),
                    t : t.clone() },
            Lambda::BinExpr { b, l, r, t } => 
                Lambda::BinExpr { b : *b, l : Rc::new(l.eval()), r : Rc::new(r.eval()), t : t.clone() }
        }
    }
}

impl Into<i64> for Lambda {
    fn into(self) -> i64 {
        match self {
            Lambda::IntLiteral { i, t: _ } => i,
            _ => panic!()
        }
    }
}

impl Into<i64> for &Lambda {
    fn into(self) -> i64 {
        match self {
            Lambda::IntLiteral { i, t: _ } => *i,
            _ => panic!()
        }
    }
}

impl Into<f64> for Lambda {
    fn into(self) -> f64 {
        match self {
            Lambda::FloatLiteral { f, t: _ } => f,
            _ => panic!()
        }
    }
}

impl Into<f64> for &Lambda {
    fn into(self) -> f64 {
        match self {
            Lambda::FloatLiteral { f, t: _ } => *f,
            _ => panic!()
        }
    }
}

impl Into<bool> for Lambda {
    fn into(self) -> bool {
        match self {
            Lambda::True { t: _ } => true,
            Lambda::False { t: _ } => false,
            _ => panic!()
        }
    }
}

impl Into<bool> for &Lambda {
    fn into(self) -> bool {
        match self {
            Lambda::True { t: _ } => true,
            Lambda::False { t: _ } => false,
            _ => panic!()
        }
    }
}

impl From<bool> for Lambda {
    fn from(b : bool) -> Lambda {
        match b {
            true => Lambda::True { t : Type::Bool },
            false => Lambda::False { t : Type::Bool }
        }
    }
}

impl std::ops::Add for Lambda {
    type Output = Lambda;

    fn add(self, rhs : Lambda) -> Lambda {
        match self.t() {
            Type::Integer => {
                let tt = self.t().clone();
                let x : i64 = self.into();
                let y : i64 = rhs.into();
                Lambda::IntLiteral { i : x + y, t : tt }
            },
            Type::Float => {
                let tt = self.t().clone();
                let x : f64 = self.into();
                let y : f64 = rhs.into();
                Lambda::FloatLiteral { f : x + y, t : tt }
            },
            _ => panic!()
        }
    }
}

impl std::ops::Sub for Lambda {
    type Output = Lambda;

    fn sub(self, rhs : Lambda) -> Lambda {
        match self.t() {
            Type::Integer => {
                let tt = self.t().clone();
                let x : i64 = self.into();
                let y : i64 = rhs.into();
                Lambda::IntLiteral { i : x - y, t : tt }
            },
            Type::Float => {
                let tt = self.t().clone();
                let x : f64 = self.into();
                let y : f64 = rhs.into();
                Lambda::FloatLiteral { f : x - y, t : tt }
            },
            _ => panic!()
        }
    }
}

impl std::ops::Mul for Lambda {
    type Output = Lambda;

    fn mul(self, rhs : Lambda) -> Lambda {
        match self.t() {
            Type::Integer => {
                let tt = self.t().clone();
                let x : i64 = self.into();
                let y : i64 = rhs.into();
                Lambda::IntLiteral { i : x * y, t : tt }
            },
            Type::Float => {
                let tt = self.t().clone();
                let x : f64 = self.into();
                let y : f64 = rhs.into();
                Lambda::FloatLiteral { f : x * y, t : tt }
            },
            _ => panic!()
        }
    }
}

impl std::ops::Div for Lambda {
    type Output = Lambda;

    fn div(self, rhs : Lambda) -> Lambda {
        match self.t() {
            Type::Integer => {
                let tt = self.t().clone();
                let x : i64 = self.into();
                let y : i64 = rhs.into();
                Lambda::IntLiteral { i : x / y, t : tt }
            },
            Type::Float => {
                let tt = self.t().clone();
                let x : f64 = self.into();
                let y : f64 = rhs.into();
                Lambda::FloatLiteral { f : x / y, t : tt }
            },
            _ => panic!()
        }
    }
}

impl PartialEq for Lambda {
    fn eq(&self, other: &Lambda) -> bool {
        match self.t() {
            Type::Integer => {
                let x : i64 = self.into();
                let y : i64 = other.into();
                x == y
            },
            Type::Float => {
                let x : f64 = self.into();
                let y : f64 = other.into();
                x == y
            },
            _ => panic!()
        }
    }
}

impl PartialOrd for Lambda {
    fn partial_cmp(&self, other: &Lambda) -> Option<Ordering> {
        match self.t() {
            Type::Integer => {
                let x : i64 = self.into();
                let y : i64 = other.into();
                if x < y { Some (Ordering::Less) }
                else if x > y { Some (Ordering::Greater) }
                else { Some (Ordering::Equal) }
            },
            Type::Float => {
                let x : f64 = self.into();
                let y : f64 = other.into();
                if x < y { Some (Ordering::Less) }
                else if x > y { Some (Ordering::Greater) }
                else { Some (Ordering::Equal) }
            },
            _ => panic!()
        }
    }
}


impl Lambda {
    pub fn eval(&self) -> Lambda {
        match self {
            Lambda::IntLiteral { i : _, t : _ } => self.clone(),
            Lambda::FloatLiteral { f : _, t : _ } => self.clone(),
            Lambda::True { t : _ } => self.clone(),
            Lambda::False { t : _ } => self.clone(),
            Lambda::Var { v : _, t : _ } => self.clone(),
            Lambda::Tuple { tup, t } => 
                Lambda::Tuple { tup : tup.iter().map(|x| x.eval()).collect(), t : t.clone() },
            Lambda::Index { i, e, t : _ } => {
                match e.eval() {
                    Lambda::Tuple { tup, t : _ } => tup[*i as usize].eval(),
                    _ => panic!()
                }
            },
            Lambda::Abs { x : _, e : _, t : _ } => self.clone(),
            Lambda::App { lhs, rhs, t : _ } => {
                match lhs.eval() {
                    Lambda::Abs { x, e, t : _ } => e.substitute(&x, rhs.eval()),
                    _ => panic!()
                }
            },
            Lambda::IfExpr { c, e1, e2, t : _ } => {
                match c.eval() {
                    Lambda::True { t : _ } => e1.eval(),
                    Lambda::False { t : _ } => e2.eval(),
                    _ => panic!()
                }
            },
            Lambda::BinExpr { b, l, r, t } => 
                b.eval(t.clone(), l.eval(), r.eval())
        }
    }
}

