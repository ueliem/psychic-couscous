use rand;
use rand::distributions::Distribution;
use std::rc::Rc;
use std::borrow::Borrow;

use super::symgen;
use super::syntax;
use super::ast;
use super::values::*;
use super::lambda::*;
use super::machineterm;
use super::store;

#[derive(Debug)]
pub struct Simulator {
    pub time : f64,
    rngdist : rand::distributions::Uniform<f64>,
    rng : rand::rngs::ThreadRng,
    pub s : store::Store,
    mt : Rc<machineterm::MachineTerm>
}

impl<'a> Simulator {
    pub fn new() -> Simulator {
        Simulator {
            time: 0.0, 
            rngdist : rand::distributions::Uniform::new(0.0, 1.0),
            rng : rand::thread_rng(),
            s: store::Store::new(), 
            mt : Rc::new(machineterm::MachineTerm::empty())
        }
    }
    fn construct<'b>(&mut self, proc : &ast::Process, term : Rc<machineterm::MachineTerm>) -> Rc<machineterm::MachineTerm> {
        match &*term {
            &machineterm::MachineTerm::TopRestriction (ref c, r, ref mt) => 
                Rc::new(machineterm::MachineTerm::TopRestriction (c.clone(), r, self.construct (proc, mt.clone()))),
            &machineterm::MachineTerm::SummList (ref sl) => {
                match proc {
                    ast::Process::Restriction (ref c, r, ref p) => {
                        let fresh : String = symgen::next();
                        let pp = &mut p.substitute(&c, fresh.as_str());
                        return Rc::new(machineterm::MachineTerm::TopRestriction
                            (fresh,
                            *r,
                            self.construct(pp, Rc::new(machineterm::MachineTerm::SummList (sl.clone())))));
                    },
                    ast::Process::LetVal (ref v, ref l, ref p) => {
                        self.construct(&p.substitute(v, l.eval()), term)
                    },
                    ast::Process::Parallel (p1, p2) => {
                        let mt1 = self.construct (p2, Rc::new(machineterm::MachineTerm::SummList (sl.clone())));
                        return self.construct(p1, mt1);
                    },
                    ast::Process::Summation (apvec) => {
                        let newsumm = Rc::new(machineterm::Summ (None, apvec.clone()));
                        let counts = newsumm.get_act_counts();
                        self.s.add_counts(counts);
                        let mut v = vec![newsumm];
                        v.extend_from_slice(sl);
                        return Rc::new(machineterm::MachineTerm::SummList (v));
                    },
                    ast::Process::Instance (ref name, params) => {
                        self.s.create(name.to_string());
                        let (pats, p) = match self.s.defs.get(name) {
                            Some ((pats, p)) => {
                                (pats, pats.iter().zip(params.iter()).fold(p.clone(), |p1, (pat, v)| Rc::new(p1.replace(pat, v))))
                            },
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
                    ast::Process::Repetition (i, p) => {
                        (0..*i).fold(term, |acc, _x| self.construct(p, acc))
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
                    match (*d).borrow() {
                        syntax::Declaration::NewChannel (ref c, r) => self.s.add_channel((*c).borrow(), *r),
                        syntax::Declaration::Run (p) => toplevelproc.push(p),
                        syntax::Declaration::Val (_, _) => panic!(),
                        syntax::Declaration::Def (n, params, ref d) => {
                            self.s.defs.insert(n.to_string(), (params.to_vec(), Rc::new(ast::Process::from((*d).borrow()))));
                            self.s.instance_counts.insert(n.to_string(), 0);
                        }
                    }
                }
                self.mt = toplevelproc.iter().rev().fold(Rc::new(machineterm::MachineTerm::SummList(Vec::new())), |acc, x| {
                    let p : &syntax::Process = (**x).borrow();
                    self.construct(&ast::Process::from(p), acc)
                });
            }
        }
    }
    fn gillespie(&self, n1 : f64, n2 : f64) -> (String, f64) {
        let activities = self.s.activities();
        let a0 = activities.iter().fold(0.0, |acc, (_v, a)| acc + a);
        let tau = (1.0 / a0) * (1.0 / n1).ln();
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
            if test > lowerbnd && test <= upperbnd {
                return (activities[i].0.to_string(), tau);
            }
        }
        panic!();
    }
    pub fn reduce(&mut self) {
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
            let (isli, islj) = self.mt.seek(ast::Act::Input(nextchan.clone()), inputindex);
            let si = Rc::get_mut(&mut self.mt).unwrap().take_summ(isli);
            let icounts_remove = si.get_act_counts();
            self.s.remove_counts(icounts_remove);

            let outcount = match self.s.chans.get(&nextchan) {
                Some (c) => c.outcount,
                None => panic!()
            };
            let outputindex = self.rng.gen_range(0, outcount);
            let (osli, oslj) = self.mt.seek(ast::Act::Output(nextchan.clone()), outputindex);
            let so = Rc::get_mut(&mut self.mt).unwrap().take_summ(osli);
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
    }
}


