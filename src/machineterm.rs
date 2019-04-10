use std::rc::Rc;
use std::collections::BTreeMap;
use super::ast;

#[derive(Debug)]
pub struct Summ (pub Option<String>, pub Rc<Vec<(ast::Act, Rc<ast::Process>)>>);

#[derive(Debug)]
pub enum MachineTerm {
    TopRestriction (String, f64, Rc<MachineTerm>),
    SummList (Vec<Rc<Summ>>)
}

impl Summ {
    pub fn new(origin : Option<String>, s : Rc<Vec<(ast::Act, Rc<ast::Process>)>>) -> Summ {
        Summ (origin, s.clone())
    }
    pub fn index(&self, i : usize) -> (ast::Act, Rc<ast::Process>) {
        self.1[i].clone()
    }
    pub fn get_act_counts<'a>(&'a self) -> BTreeMap<&'a str, (usize, usize, usize)> {
        let mut counts : BTreeMap<&'a str, (usize, usize, usize)> = BTreeMap::new();
        for (a, p) in self.1.iter() {
            match a {
                ast::Act::Input (c) => {
                    counts.entry(c).or_insert((0, 0, 0)).0 += 1;
                },
                ast::Act::Output (c) => {
                    counts.entry(c).or_insert((0, 0, 0)).1 += 1;
                },
            }
        }
        for (_, c) in counts.iter_mut() {
            c.2 = c.0 * c.1;
        }
        return counts;
    }
}

impl MachineTerm {
    pub fn empty() -> MachineTerm {
        MachineTerm::SummList (Vec::new())
    }
    pub fn is_summlist(&self) -> bool {
        match self {
            &MachineTerm::TopRestriction(_, _, _) => false,
            &MachineTerm::SummList (_) => true
        }
    }
    pub fn is_restr(&self) -> bool {
        match self {
            &MachineTerm::TopRestriction(_, _, _) => true,
            &MachineTerm::SummList (_) => false
        }
    }
    pub fn seek(&self, elem : ast::Act, count : usize) -> (usize, usize) {
        let mut c = count;
        match self {
            &MachineTerm::TopRestriction(_, _, _) => panic!(),
            &MachineTerm::SummList (ref sl) => {
                for i in 0..sl.len() {
                    for j in 0..sl[i].1.len() {
                        match (sl[i].1[j].0.clone(), elem.clone()) {
                            (ast::Act::Input (n1), ast::Act::Input (n2)) |
                            (ast::Act::Output (n1), ast::Act::Output (n2)) => {
                                if n1 == n2 {
                                    if c == 0 {
                                        // println!("located");
                                        return (i, j);
                                    }
                                    else {
                                        c -= 1;
                                    }
                                }
                                else {
                                    continue;
                                }
                            },
                            _ => continue
                        }
                    }
                }
            }
        }
        panic!();
    }
    pub fn take_chan(&self) -> (String, f64) {
        match self {
            MachineTerm::TopRestriction(n, r, _) => (n.clone(), *r),
            MachineTerm::SummList (_) => panic!()
        }
    }
    pub fn take_inner(&self) -> Rc<MachineTerm> {
        match self {
            MachineTerm::TopRestriction(_, _, t) => t.clone(),
            MachineTerm::SummList (_) => panic!()
        }
    }
    pub fn take_summ(&mut self, i : usize) -> Rc<Summ> {
        match self {
            &mut MachineTerm::TopRestriction(_, _, _) => panic!(),
            &mut MachineTerm::SummList (ref mut sl) => {
                sl.remove(i).clone()
            }
        }
    }
}

