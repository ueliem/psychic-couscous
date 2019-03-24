use super::ast;

#[derive(Debug)]
pub enum MachineTerm {
    TopRestriction (String, Box<MachineTerm>),
    SummList (Box<Vec<Box<ast::Summ>>>)
}


