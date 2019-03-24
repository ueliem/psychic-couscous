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
    s : store::Store,
    mt : Box<machineterm::MachineTerm>
}

impl Simulator {
    pub fn new() -> Simulator {
        Simulator {
            time: 0.0, 
            s: store::Store::new(), 
            mt : box machineterm::MachineTerm::SummList(box Vec::new()),
            symgen : SymbolGenerator::new()
        }
    }
    fn construct (&mut self, proc : &ast::Process, term : Box<machineterm::MachineTerm>) -> Box<machineterm::MachineTerm> {
        match term {
            box machineterm::MachineTerm::TopRestriction (c, mt) => 
                box machineterm::MachineTerm::TopRestriction (c.clone(), self.construct (proc, mt)),
            box machineterm::MachineTerm::SummList (box mut sl) => {
                match proc {
                    ast::Process::Restriction (ref c, ref p) => {
                        let fresh = self.symgen.next();
                        return box machineterm::MachineTerm::TopRestriction 
                            (fresh.clone(),
                             self.construct(&mut p.substitute(&c, &fresh), box machineterm::MachineTerm::SummList (box sl)));
                    },
                    ast::Process::Parallel (box p1, box p2) => {
                        let mt1 = self.construct (p2, box machineterm::MachineTerm::SummList (box sl));
                        return self.construct (p1, mt1);
                    },
                    ast::Process::Summation (box apvec) => {
                        // sl.push((**apvec).clone());
                        // sl.append(&mut vec![apvec.clone()]);
                        // let mut v = vec![apvec];
                        // # v.append(&mut sl);
                        sl.append(&mut vec![box apvec.clone()]);
                        return box machineterm::MachineTerm::SummList (box sl);
                    },
                    ast::Process::Replication (a, p) => {
                        self.construct (
                            &mut ast::Process::Summation (
                                box vec![(a.clone(), box ast::Process::Parallel ((*p).clone(), box proc.clone()))]), 
                            box machineterm::MachineTerm::SummList (box sl))
                    },
                    ast::Process::Termination => box machineterm::MachineTerm::SummList (box sl)
                }
            }
        }
    }
    pub fn load(&mut self, p : &ast::Process) {
        let mtc = self.construct(&mut p.clone(), box machineterm::MachineTerm::SummList(box Vec::new()));
        self.mt = mtc;
    }
    pub fn reduce(&mut self) {
        self.mt = match self.mt {
            box machineterm::MachineTerm::TopRestriction(c, box t) => {
                self.s.add_channel(c.clone());
                box t
            },
            box machineterm::MachineTerm::SummList (ref sl) => {
                panic!();
            }
        }
    }
}


