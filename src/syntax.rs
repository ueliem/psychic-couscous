use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Declaration {
    NewChannel (String, f64),
    Run (Rc<Process>)
}

pub type Summ = Vec<(Act, Rc<Process>)>;

#[derive(Clone, Debug)]
pub enum Act {
    Input (String, String),
    Output (String, String)
}
#[derive(Clone, Debug)]
pub enum Process {
    Parallel (Rc<Vec<Rc<Process>>>),
    Action (Act, Rc<Process>),
    Choice (Rc<Summ>),
    // Instance (String, 
    Replication (Act, Rc<Process>),
    NestedDecl (Rc<Vec<Rc<Declaration>>>, Rc<Process>),
    Termination
}

#[derive(Clone, Debug)]
pub enum Program {
    Prog (Rc<Vec<Rc<Declaration>>>)
}


