use std::rc::Rc;
use std::convert::From;
use std::borrow::Borrow;

use super::syntax;

pub trait Substitutable {
    fn substitute (&self, src : &String, dest : &String) -> Self;
}

pub type Summ = Vec<(Act, Rc<Process>)>;

#[derive(Clone, Debug)]
pub enum Act {
    Input (String, String),
    Output (String, String)
}
#[derive(Clone, Debug)] pub enum Process {
    Restriction (String, Rc<Process>),
    Parallel (Rc<Process>, Rc<Process>),
    Summation (Rc<Summ>),
    Replication (Act, Rc<Process>),
    Termination
}

impl Substitutable for Act {
    fn substitute(&self, src : &String, dest : &String) -> Act {
        match self {
            Act::Input (c, v) => 
                Act::Input (
                    if *c == *src {dest.clone()} else {c.clone()}, 
                    v.clone()),
            Act::Output (c, v) => 
                Act::Output (
                    if *c == *src {dest.clone()} else {c.clone()}, 
                    if *v == *src {dest.clone()} else {v.clone()})
        }
    }
}

impl Substitutable for Process {
    fn substitute(&self, src : &String, dest : &String) -> Process {
        match self {
            Process::Restriction (c, ref p) => 
                Process::Restriction (c.clone(), Rc::new(p.substitute(src, dest))),
            Process::Parallel (ref p1, ref p2) => 
                Process::Parallel (
                    Rc::new(p1.substitute(src, dest)), 
                    Rc::new(p2.substitute(src, dest))),
            Process::Summation (apvec) => 
                Process::Summation (
                    Rc::new(apvec.clone().iter()
                    .map(|(a, p)| (a.substitute(src, dest), Rc::new(p.substitute(src, dest))))
                    .collect())),
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
            syntax::Act::Input (c, v) => Act::Input (c.clone(), v.clone()),
            syntax::Act::Output (c, v) => Act::Output (c.clone(), v.clone())
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
            syntax::Process::Replication (ref a, ref p) => {
                let pb : &syntax::Process = p.borrow();
                let pbb : Process = Process::from(pb);
                Process::Replication (a.into(), Rc::new(pbb))
            },
            /* syntax::Process::NestedDecl (ref dl, ref p) => {
                let dlb : &Vec<Rc<syntax::Declaration>> = dl.borrow();
                let diter = dlb.iter().rev();
            }, */
            syntax::Process::Termination => Process::Termination,
            _ => panic!()
        }
    }
}

impl From<&syntax::Program> for Process {
    fn from(syn : &syntax::Program) -> Process {
        match *syn {
            syntax::Program::Prog (ref dl) => {
                let dlb : &Vec<Rc<syntax::Declaration>> = dl.borrow();
                let diter = dlb.iter().rev();
                panic!();
                // syntax::Declaration::NewChannel (c, r) => Process::Restriction (c, panic!()),
                /*syntax::Declaration::Run (ref p) => {
                    let pb : &syntax::Process = p.borrow();
                    Process::from(pb)
                }, */
                // _ => panic!()
            }
        }
    }
}

