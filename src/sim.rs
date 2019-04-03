use rand;
use rand::distributions::Distribution;
use std::rc::Rc;
use std::borrow::Borrow;

use super::syntax;
use super::ast;
use super::ast::Substitutable;
use super::machineterm;
use super::store;

#[derive(Debug)]
struct SymbolGenerator {
    counter : usize
}

impl SymbolGenerator {
    pub fn new() -> SymbolGenerator {
        SymbolGenerator {counter : 0}
    }
    pub fn next(&mut self) -> String {
        let n = self.counter.to_string();
        self.counter = self.counter + 1;
        return n;
    }
}

#[derive(Debug)]
pub struct Simulator {
    time : f64,
    symgen : SymbolGenerator,
    rngdist : rand::distributions::Uniform<f64>,
    rng : rand::rngs::ThreadRng,
    s : store::Store,
    mt : Rc<machineterm::MachineTerm>
}

impl Simulator {
    pub fn new() -> Simulator {
        Simulator {
            time: 0.0, 
            symgen : SymbolGenerator::new(),
            rngdist : rand::distributions::Uniform::new(0.0, 1.0),
            rng : rand::thread_rng(),
            s: store::Store::new(), 
            mt : Rc::new(machineterm::MachineTerm::SummList(Rc::new(Vec::new())))
        }
    }
    fn construct (&mut self, proc : &ast::Process, term : Rc<machineterm::MachineTerm>) -> Rc<machineterm::MachineTerm> {
        let e = &*term;
        match e {
            machineterm::MachineTerm::TopRestriction (c, mt) => 
                Rc::new(machineterm::MachineTerm::TopRestriction (c.clone(), self.construct (proc, mt.clone()))),
            machineterm::MachineTerm::SummList (sl) => {
                match proc {
                    ast::Process::Restriction (ref c, ref p) => {
                        let fresh = self.symgen.next();
                        return Rc::new(machineterm::MachineTerm::TopRestriction 
                            (fresh.clone(),
                             self.construct(&mut p.substitute(&c, &fresh), Rc::new(machineterm::MachineTerm::SummList (sl.clone())))));
                    },
                    ast::Process::Parallel (p1, p2) => {
                        let mt1 = self.construct (p2, Rc::new(machineterm::MachineTerm::SummList (sl.clone())));
                        return self.construct (p1, mt1);
                    },
                    ast::Process::Summation (apvec) => {
                        let mut v = vec![apvec.clone()];
                        v.extend_from_slice(sl);
                        return Rc::new(machineterm::MachineTerm::SummList (Rc::new(v)));
                    },
                    ast::Process::Replication (a, p) => {
                        self.construct (
                            &mut ast::Process::Summation (
                                Rc::new(vec![(a.clone(), Rc::new(ast::Process::Parallel ((*p).clone(), Rc::new(proc.clone()))))])),
                            Rc::new(machineterm::MachineTerm::SummList (sl.clone())))
                    },
                    ast::Process::Termination => Rc::new(machineterm::MachineTerm::SummList (sl.clone()))
                }
            }
        }
    }
    pub fn load(&mut self, p : &syntax::Program) {
        match *p {
            syntax::Program::Prog(ref dl) => {
                let mut toplevelproc = Vec::new();
                for d in dl.iter() {
                    println!("{:?}", d);
                    match (*d).borrow() {
                        syntax::Declaration::NewChannel (ref c, r) => self.s.add_channel(c, *r),
                        syntax::Declaration::Run (p) => toplevelproc.push(p)
                    }
                }
                self.mt = toplevelproc.iter().rev().fold(Rc::new(machineterm::MachineTerm::SummList(Rc::new(Vec::new()))), |acc, x| {
                    let p : &syntax::Process = (**x).borrow();
                    self.construct(&ast::Process::from(p), acc)
                });
            }
        }
    }
    fn gillespie(&mut self) -> (String, f64) {
        let activities = self.s.activities();
        let a0 = activities.iter().fold(0.0, |acc, (v, a)| acc + a);
        let n1 = self.rngdist.sample(&mut self.rng);
        let n2 = self.rngdist.sample(&mut self.rng);
        let tau = (1.0 / a0) * (1.0 / n1).ln();

        for i in 1..activities.len() {
            // let lowerbnd = 
            // let upperbnd = 
            println!("{:?} {:?}", activities[i].0, activities[i].1);
        }

        return ("".to_string(), tau);
    }
    pub fn reduce(&mut self) {
        let e = &*self.mt;
        match e {
            machineterm::MachineTerm::TopRestriction(c, t) => {
                self.s.add_channel(c, 0.0);
                self.mt = t.clone();
            },
            machineterm::MachineTerm::SummList (sl) => {
                self.gillespie();
                panic!();
            }
        }
    }
}


