use std::rc::Rc;
use std::convert::From;
use std::borrow::Borrow;

use super::syntax;

pub trait Substitutable {
    fn substitute (&self, src : &str, dest : &str) -> Self;
}

pub type Summ = Vec<(Act, Rc<Process>)>;

#[derive(Clone, Debug)]
pub enum Act {
    Input (String),
    Output (String)
}
#[derive(Clone, Debug)] pub enum Process {
    Restriction (String, f64, Rc<Process>),
    Parallel (Rc<Process>, Rc<Process>),
    Summation (Rc<Summ>),
    Instance (String),
    Repitition (usize, Rc<Process>),
    Replication (Act, Rc<Process>),
    Termination
}

impl Substitutable for Act {
    fn substitute(&self, src : &str, dest : &str) -> Act {
        match self {
            Act::Input (c) => 
                Act::Input (if *c == *src {dest.to_string()} else {c.to_string()}),
            Act::Output (c) => 
                Act::Output (if *c == *src {dest.to_string()} else {c.to_string()})
        }
    }
}

impl Substitutable for Process {
    fn substitute(&self, src : &str, dest : &str) -> Process {
        match self {
            Process::Restriction (c, r, ref p) => 
                Process::Restriction (c.clone(), *r, Rc::new(p.substitute(src, dest))),
            Process::Parallel (ref p1, ref p2) => 
                Process::Parallel (
                    Rc::new(p1.substitute(src, dest)), 
                    Rc::new(p2.substitute(src, dest))),
            Process::Summation (apvec) => 
                Process::Summation (
                    Rc::new(apvec.clone().iter()
                    .map(|(a, p)| (a.substitute(src, dest), Rc::new(p.substitute(src, dest))))
                    .collect())),
            Process::Instance (name) => {
                Process::Instance (name.to_string())
            },
            Process::Repitition (i, ref p) => 
                Process::Repitition (*i, Rc::new(p.substitute(src, dest))),
            Process::Replication (a, ref p) =>
                Process::Replication (
                    a.substitute(src, dest),
                    Rc::new(p.substitute(src, dest))),
            Process::Termination => Process::Termination
        }
    }
}

impl From<&syntax::Act> for Act {
    fn from(syn : &syntax::Act) -> Act {
        match syn {
            syntax::Act::Input (c) => Act::Input (c.clone()),
            syntax::Act::Output (c) => Act::Output (c.clone())
        }
    }
}

impl From<&syntax::Process> for Process {
    fn from(syn : &syntax::Process) -> Process {
        match *syn {
            syntax::Process::Parallel (ref p) => {
                let pborrow : &Vec<Rc<syntax::Process>> = p.borrow();
                let mut piter : std::iter::Rev<std::slice::Iter<Rc<syntax::Process>>> = pborrow.iter().rev();
                let first : &syntax::Process = piter.next().unwrap().borrow();
                let mut current : Process = Process::from(first);
                for elem in piter {
                    let e : &syntax::Process = (*elem).borrow();
                    current = Process::Parallel (Rc::new(Process::from(e)), Rc::new(current));
                }
                return current;
            },
            syntax::Process::Action (ref a, ref p) => {
                let pb : &syntax::Process = p.borrow();
                Process::Summation (Rc::new(vec![(a.into(), Rc::new(Process::from(pb)))]))
            },
            syntax::Process::Choice (ref c) => {
                let pborrow : &Vec<(syntax::Act, Rc<syntax::Process>)> = c.borrow();
                let cnew = pborrow.iter().rev().map(|(a, p)| {
                    let pb : &syntax::Process = p.borrow();
                    let pbb : Process = Process::from(pb);
                    (a.into(), Rc::new(pbb))
                });
                return Process::Summation (Rc::new(cnew.collect()));
            },
            syntax::Process::Instance (ref name) => {
                Process::Instance (name.to_string())
            },
            syntax::Process::Repitition (i, ref p) => {
                Process::Repitition (i, Rc::new(Process::from(p.borrow())))
            },
            syntax::Process::Replication (ref a, ref p) => {
                let pb : &syntax::Process = p.borrow();
                let pbb : Process = Process::from(pb);
                Process::Replication (a.into(), Rc::new(pbb))
            },
            syntax::Process::NestedDecl (ref dl, ref p) => {
                let dlb : &Vec<Rc<syntax::Declaration>> = dl.borrow();
                let diter = dlb.iter().rev();
                let mut current = Process::from((*p).borrow());
                for d in diter {
                    match (*d).borrow() {
                        syntax::Declaration::NewChannel (ref c, r) => {
                            current = Process::Restriction (c.to_string(), *r, Rc::new(current));
                        },
                        syntax::Declaration::Run (ref pr) => {
                            current = Process::Parallel (Rc::new(Process::from((*pr).borrow())), Rc::new(current));
                        },
                        syntax::Declaration::Def (ref n, ref p) => {
                            panic!();
                        }
                    }
                }
                return current;
            },
            syntax::Process::Termination => Process::Termination
        }
    }
}

