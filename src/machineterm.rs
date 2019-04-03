use std::rc::Rc;
use super::ast;

#[derive(Debug)]
pub enum MachineTerm {
    TopRestriction (String, Rc<MachineTerm>),
    SummList (Rc<Vec<Rc<ast::Summ>>>)
}


