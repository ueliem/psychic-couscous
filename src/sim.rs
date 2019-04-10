use rand;
use rand::distributions::Distribution;
use std::rc::Rc;
use std::borrow::Borrow;
use std::collections::BTreeMap;

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
    pub time : f64,
    symgen : SymbolGenerator,
    rngdist : rand::distributions::Uniform<f64>,
    rng : rand::rngs::ThreadRng,
    pub s : store::Store,
    mt : Rc<machineterm::MachineTerm>
}

impl<'a> Simulator {
    pub fn new() -> Simulator {
        Simulator {
            time: 0.0, 
            symgen : SymbolGenerator::new(),
            rngdist : rand::distributions::Uniform::new(0.0, 1.0),
            rng : rand::thread_rng(),
            s: store::Store::new(), 
            mt : Rc::new(machineterm::MachineTerm::empty())
        }
    }
    fn construct<'b>(&mut self, proc : &ast::Process, term : Rc<machineterm::MachineTerm>) -> Rc<machineterm::MachineTerm> {
        // let e = &*term;
        match &*term {
            &machineterm::MachineTerm::TopRestriction (ref c, r, ref mt) => 
                Rc::new(machineterm::MachineTerm::TopRestriction (c.clone(), r, self.construct (proc, mt.clone()))),
            &machineterm::MachineTerm::SummList (ref sl) => {
                match proc {
                    ast::Process::Restriction (ref c, r, ref p) => {
                        let fresh : String = self.symgen.next();
                        let pp = &mut p.substitute(&c, &fresh);
                        return Rc::new(machineterm::MachineTerm::TopRestriction
                            (fresh,
                            *r,
                            self.construct(pp, Rc::new(machineterm::MachineTerm::SummList (sl.clone())))));
                    },
                    ast::Process::Parallel (p1, p2) => {
                        let mt1 = self.construct (p2, Rc::new(machineterm::MachineTerm::SummList (sl.clone())));
                        return self.construct (p1, mt1);
                    },
                    ast::Process::Summation (apvec) => {
                        let newsumm = Rc::new(machineterm::Summ (None, apvec.clone()));
                        let counts = newsumm.get_act_counts();
                        self.s.add_counts(counts);
                        let mut v = vec![newsumm];
                        v.extend_from_slice(sl);
                        return Rc::new(machineterm::MachineTerm::SummList (v));
                    },
                    ast::Process::Instance (ref name) => {
                        self.s.create(name.to_string());
                        let p = match self.s.defs.get(name) {
                            Some (p) => p,
                            None => panic!()
                        };
                        match p.borrow() {
                            ast::Process::Summation (apvec) => {
                                let newsumm = Rc::new(machineterm::Summ (Some (name.clone()), apvec.clone()));
                                let counts = newsumm.get_act_counts();
                                self.s.add_counts(counts);
                                let mut v = vec![newsumm];
                                v.extend_from_slice(sl);
                                return Rc::new(machineterm::MachineTerm::SummList (v));
                            }
                            _ => self.construct(&p.clone(), term)
                        }
                    },
                    ast::Process::Repitition (i, p) => {
                        (0..*i).fold(term, |acc, x| self.construct(p, acc))
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
    pub fn load(&mut self, p : &'a syntax::Program) {
        match *p {
            syntax::Program::Prog(ref decs) => {
                let mut toplevelproc = Vec::new();
                for d in decs.iter() {
                    // println!("{:?}", d);
                    match (*d).borrow() {
                        syntax::Declaration::NewChannel (ref c, r) => self.s.add_channel((*c).borrow(), *r),
                        syntax::Declaration::Run (p) => toplevelproc.push(p),
                        syntax::Declaration::Def (n, ref d) => {
                            self.s.defs.insert(n.to_string(), Rc::new(ast::Process::from((*d).borrow())));
                            self.s.instance_counts.insert(n.to_string(), 0);
                        }
                    }
                }
                self.mt = toplevelproc.iter().rev().fold(Rc::new(machineterm::MachineTerm::SummList(Vec::new())), |acc, x| {
                    let p : &syntax::Process = (**x).borrow();
                    // println!("p {:?}", p);
                    self.construct(&ast::Process::from(p), acc)
                });
            }
        }
        // self.init_chan_counts();
    }
    fn gillespie(&self, n1 : f64, n2 : f64) -> (String, f64) {
        let activities = self.s.activities();
        let a0 = activities.iter().fold(0.0, |acc, (v, a)| acc + a);
        let tau = (1.0 / a0) * (1.0 / n1).ln();

        // println!("{} {}", a0, tau);
        // println!("{:?}", activities);

        for i in 0..activities.len() {
            let test = a0 * n2;
            let lowerbnd = 
                if i == 0 {
                    0.0
                }
                else {
                    activities[0..(i)].iter().fold(0.0, |acc, x| acc + x.1)
                };
            let upperbnd = 
                if i >= activities.len() {
                    test
                }
                else {
                    activities[0..i+1].iter().fold(0.0, |acc, x| acc + x.1)
                };
            // println!("{:?}", activities[0..i].iter());
            // println!("{} {} {}", lowerbnd, test, upperbnd);
            if test > lowerbnd && test <= upperbnd {
                return (activities[i].0.to_string(), tau);
            }
        }
        panic!();
    }
    pub fn reduce(&mut self) {
        // self.mt = match *e {
            // machineterm::MachineTerm::TopRestriction(ref c, r, ref t) => {
        let is_restr = self.mt.is_restr();
        let is_summlist = self.mt.is_summlist();
        if is_restr {
            let (c, r) = self.mt.take_chan();
            self.s.add_channel(&c.to_string(), r);
            self.mt = self.mt.take_inner();
        }
        else if is_summlist {
            use rand::Rng;
            let n1 = self.rngdist.sample(&mut self.rng);
            let n2 = self.rngdist.sample(&mut self.rng);
            let (nextchan, tau) = self.gillespie(n1, n2);
            let incount = match self.s.chans.get(&nextchan) {
                Some (c) => c.incount,
                None => panic!()
            };
            let inputindex = self.rng.gen_range(0, incount);
            // println!("{} {} {}", nextchan, tau, inputindex);
            let (isli, islj) = self.mt.seek(ast::Act::Input(nextchan.clone()), inputindex);
            // println!("{} {}", isli, islj);
            // let e = Rc::get_mut(&mut self.mt).unwrap();
            // let mut si = e.take_summ(isli);
            let si = Rc::get_mut(&mut self.mt).unwrap().take_summ(isli);
            // println!("{:?}", si);
            let icounts_remove = si.get_act_counts();
            self.s.remove_counts(icounts_remove);

            let outcount = match self.s.chans.get(&nextchan) {
                Some (c) => c.outcount,
                None => panic!()
            };
            let outputindex = self.rng.gen_range(0, outcount);
            let (osli, oslj) = self.mt.seek(ast::Act::Output(nextchan.clone()), outputindex);
            let so = Rc::get_mut(&mut self.mt).unwrap().take_summ(osli);
            // println!("{:?}", so);
            let ocounts_remove = so.get_act_counts();
            self.s.remove_counts(ocounts_remove);

            let ip = si.index(islj);
            let op = so.index(oslj);

            self.mt = self.construct (&ip.1, self.mt.clone());
            self.mt = self.construct (&op.1, self.mt.clone());

            match *si {
                machineterm::Summ (Some(ref name), _) => {
                    self.s.destroy(name.to_string());
                },
                _ => ()
            }
            match *so {
                machineterm::Summ (Some(ref name), _) => {
                    self.s.destroy(name.to_string());
                },
                _ => ()
            }

            self.time += tau;
        }
        // panic!();
    }
}


