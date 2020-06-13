use std::rc::Rc;

pub trait Substitutable<I> {
    fn substitute (&self, src : &str, dest : I) -> Self;
}

#[derive(Clone, Debug)]
pub enum Pattern {
    Wildcard,
    Name (String),
    Tuple (Vec<Pattern>)
}

#[derive(Clone, Debug)]
pub enum Type {
    Unit,
    TVar,
    Integer,
    Float,
    Bool,
    Channel (Option<Rc<Type>>),
    Constructor (Vec<Type>),
    Tuple,
    Function (Rc<Type>, Rc<Type>)
}

