use std::rc::Rc;
use std::convert::From;
use std::borrow::Borrow;

use diff_enum::common_fields;

use super::symgen;
use super::syntax;
use super::values::*;
use super::lambda::*;

pub type Summ = Vec<(Act, Rc<Process>)>;

#[derive(Clone, Debug)]
pub enum Act {
    Input (String),
    Output (String)
}
#[derive(Clone, Debug)] pub enum Process {
    Restriction (String, f64, Rc<Process>),
    LetVal (String, Lambda, Rc<Process>),
    Parallel (Rc<Process>, Rc<Process>),
    Summation (Rc<Summ>),
    Instance (String, Vec<Lambda>),
    Repetition (usize, Rc<Process>),
    Replication (Act, Rc<Process>),
    Termination
}

impl Substitutable<&str> for Act {
    fn substitute(&self, src : &str, dest : &str) -> Act {
        match self {
            Act::Input (c) => 
                Act::Input (if *c == *src {dest.to_string()} else {c.to_string()}),
            Act::Output (c) => 
                Act::Output (if *c == *src {dest.to_string()} else {c.to_string()})
        }
    }
}

impl Substitutable<Lambda> for Act {
    fn substitute(&self, _src : &str, _dest : Lambda) -> Act {
        match self {
            /* Act::Input (c) => 
                Act::Input (if *c == *src {dest.to_string()} else {c.to_string()}),
            Act::Output (c) => 
                Act::Output (if *c == *src {dest.to_string()} else {c.to_string()}) */
            _ => self.clone()
        }
    }
}

impl Substitutable<&str> for Process {
    fn substitute(&self, src : &str, dest : &str) -> Process {
        match self {
            Process::Restriction (c, r, ref p) => 
                Process::Restriction (c.clone(), *r, Rc::new(p.substitute(src, dest))),
            Process::LetVal (ref v, ref l, ref p) => {
                Process::LetVal (v.to_string(), l.clone(), Rc::new(p.substitute(src, dest)))
            },
            Process::Parallel (ref p1, ref p2) => 
                Process::Parallel (
                    Rc::new(p1.substitute(src, dest)), 
                    Rc::new(p2.substitute(src, dest))),
            Process::Summation (apvec) => 
                Process::Summation (
                    Rc::new(apvec.clone().iter()
                    .map(|(a, p)| (a.substitute(src, dest), Rc::new(p.substitute(src, dest))))
                    .collect())),
            Process::Instance (name, params) => {
                Process::Instance (name.to_string(), params.clone())
            },
            Process::Repetition (i, ref p) => 
                Process::Repetition (*i, Rc::new(p.substitute(src, dest))),
            Process::Replication (a, ref p) =>
                Process::Replication (
                    a.substitute(src, dest),
                    Rc::new(p.substitute(src, dest))),
            Process::Termination => Process::Termination
        }
    }
}

impl Substitutable<Lambda> for Process {
    fn substitute(&self, src : &str, dest : Lambda) -> Process {
        match self {
            Process::Restriction (c, r, ref p) => 
                Process::Restriction (c.clone(), *r, Rc::new(p.substitute(src, dest))),
            Process::LetVal (ref v, ref l, ref p) => {
                Process::LetVal (v.to_string(), l.substitute(src, dest.clone()), Rc::new(p.substitute(src, dest.clone())))
            },
            Process::Parallel (ref p1, ref p2) => 
                Process::Parallel (
                    Rc::new(p1.substitute(src, dest.clone())), 
                    Rc::new(p2.substitute(src, dest.clone()))),
            Process::Summation (apvec) => 
                Process::Summation (
                    Rc::new(apvec.clone().iter()
                    .map(|(a, p)| (a.substitute(src, dest.clone()), Rc::new(p.substitute(src, dest.clone()))))
                    .collect())),
            Process::Instance (name, params) => {
                Process::Instance (name.to_string(), params.iter().map(|x| x.substitute(src, dest.clone())).collect())
            },
            Process::Repetition (i, ref p) => 
                Process::Repetition (*i, Rc::new(p.substitute(src, dest.clone()))),
            Process::Replication (a, ref p) =>
                Process::Replication (
                    a.substitute(src, dest.clone()),
                    Rc::new(p.substitute(src, dest.clone()))),
            Process::Termination => Process::Termination
        }
    }
}

impl Substitutable<&Lambda> for Process {
    fn substitute(&self, src : &str, dest : &Lambda) -> Process {
        match self {
            Process::Restriction (c, r, ref p) => 
                Process::Restriction (c.clone(), *r, Rc::new(p.substitute(src, dest))),
            Process::LetVal (ref v, ref l, ref p) => {
                Process::LetVal (v.to_string(), l.substitute(src, dest.clone()), Rc::new(p.substitute(src, dest.clone())))
            },
            Process::Parallel (ref p1, ref p2) => 
                Process::Parallel (
                    Rc::new(p1.substitute(src, dest.clone())), 
                    Rc::new(p2.substitute(src, dest.clone()))),
            Process::Summation (apvec) => 
                Process::Summation (
                    Rc::new(apvec.clone().iter()
                    .map(|(a, p)| (a.substitute(src, dest.clone()), Rc::new(p.substitute(src, dest.clone()))))
                    .collect())),
            Process::Instance (name, params) => {
                Process::Instance (name.to_string(), params.iter().map(|x| x.substitute(src, dest.clone())).collect())
            },
            Process::Repetition (i, ref p) => 
                Process::Repetition (*i, Rc::new(p.substitute(src, dest.clone()))),
            Process::Replication (a, ref p) =>
                Process::Replication (
                    a.substitute(src, dest.clone()),
                    Rc::new(p.substitute(src, dest.clone()))),
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

fn recurse (pat : &Pattern, parent : String) -> Vec<(String, Lambda)> {
    let mut vl = Vec::new();
    match pat {
        Pattern::Wildcard => panic!(),
        Pattern::Name (n) => {
            vl.push((n.to_string(), Lambda::Var { v : parent.to_string(), t : Type::Unit }));
        }
        Pattern::Tuple (pl) => {
            for (i, p) in pl.iter().enumerate() {
                let v = symgen::next();
                vl.push((v.clone(), Lambda::Index { i : i as i64, e : Rc::new(Lambda::Var { v : parent.to_string(), t : Type::Unit }), t : Type::Unit }));
                recurse(p.borrow(), v.clone());
            }
        }
    }
    vl
}

fn destruct(pat : &Pattern, expr : Lambda, base : Process) -> Process {
    let newvar = symgen::next();
    let vl = recurse(pat, newvar.to_string());
    let mut current = base;
    for (v, l) in vl.iter().rev() {
        current = Process::LetVal (v.to_string(), l.clone(), Rc::new(current));
    }
    Process::LetVal (newvar, expr, Rc::new(current))
}

impl From<&syntax::Process> for Process {
    fn from(syn : &syntax::Process) -> Process {
        match *syn {
            syntax::Process::Restriction (ref c, r, ref p) => {
                Process::Restriction (c.to_string(), r, Rc::new(Process::from(p.borrow())))
            },
            syntax::Process::LetVal (ref pat, ref l, ref p) => {
                match pat {
                    Pattern::Wildcard => panic!(),
                    Pattern::Name (n) => Process::LetVal (n.to_string(), l.clone(), Rc::new(Process::from(p.borrow()))),
                    Pattern::Tuple (pl) => {
                        destruct(&pat.clone(), l.clone(), Process::from(p.borrow()))
                    }
                }
            },
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
            syntax::Process::Instance (ref name, ref params) => {
                Process::Instance (name.to_string(), params.clone())
            },
            syntax::Process::Repetition (i, ref p) => {
                Process::Repetition (i, Rc::new(Process::from(p.borrow())))
            },
            syntax::Process::Replication (ref a, ref p) => {
                let pb : &syntax::Process = p.borrow();
                let pbb : Process = Process::from(pb);
                Process::Replication (a.into(), Rc::new(pbb))
            },
            syntax::Process::Termination => Process::Termination
        }
    }
}

impl Process {
    pub fn replace(&self, formals : &Pattern, vals : &Lambda) -> Process {
        match (formals, vals) {
            (Pattern::Wildcard, _) => self.clone(),
            (Pattern::Name (n), l) => self.substitute(n.as_str(), l),
            (Pattern::Tuple (ref tv), Lambda::Tuple { tup, t }) => {
                tv.iter().zip(tup.iter()).fold(self.clone(), |p1, (pat, v)| p1.replace(pat, v))
            },
            (Pattern::Tuple (_), _) => panic!()
        }
    }
}

