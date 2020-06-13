use std::rc::Rc;

use super::values::*;
use super::lambda::*;

#[derive(Clone, Debug)]
pub enum Declaration {
    NewChannel (String, f64),
    Run (Rc<Process>),
    Val (Pattern, Lambda),
    Def (String, Vec<Pattern>, Rc<Process>)
}

pub type Summ = Vec<(Act, Rc<Process>)>;

#[derive(Clone, Debug)]
pub enum Act {
    Input (String),
    Output (String)
}
#[derive(Clone, Debug)]
pub enum Process {
    Restriction (String, f64, Rc<Process>),
    LetVal (Pattern, Lambda, Rc<Process>),
    Parallel (Rc<Vec<Rc<Process>>>),
    Action (Act, Rc<Process>),
    Choice (Rc<Summ>),
    Instance (String, Vec<Lambda>),
    Repetition (usize, Rc<Process>),
    Replication (Act, Rc<Process>),
    Termination
}

#[derive(Clone, Debug)]
pub enum Program {
    Prog (Rc<Vec<Rc<Declaration>>>)
}


