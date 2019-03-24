pub trait Substitutable {
    fn substitute (&self, src : &String, dest : &String) -> Self;
}

#[derive(Debug)]
pub struct Channel {
    name: String, 
    rate: f64
}

pub type Summ = Vec<(Act, Box<Process>)>;

#[derive(Clone, Debug)]
pub enum Act {
    Input (String, String),
    Output (String, String)
}

#[derive(Clone, Debug)]
pub enum Process {
    Restriction (String, Box<Process>),
    Parallel (Box<Process>, Box<Process>),
    Summation (Box<Summ>),
    Replication (Act, Box<Process>),
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
                Process::Restriction (c.clone(), Box::new(p.substitute(src, dest))),
            Process::Parallel (ref p1, ref p2) => 
                Process::Parallel (
                    Box::new(p1.substitute(src, dest)), 
                    Box::new(p2.substitute(src, dest))),
            Process::Summation (apvec) => 
                Process::Summation (
                    Box::new(apvec.clone().into_iter()
                    .map(|(a, p)| (a.substitute(src, dest), Box::new(p.substitute(src, dest))))
                    .collect())),
            Process::Replication (a, ref p) =>
                Process::Replication (
                    a.substitute(src, dest),
                    Box::new(p.substitute(src, dest))),
            Process::Termination => Process::Termination
        }
    }
}

/* TopRestriction (c, construct (proc, mt))
| construct (proc, SummList (sl)) = 
  (case proc of
    Restriction (c, p) => 
      let val freshv = newSymbol ()
        val p' = substitute (c, freshv, p)
      in
        TopRestriction (freshv, construct (p', SummList (sl)))
      end
  | Parallel (p1, p2) => construct (p1, construct (p2, SummList (sl)))
  | Summation (apl) => SummList (apl::sl)
  | Replication (a, p) => 
      construct (Summation ([(a, Parallel (p, proc))]), SummList (sl))
  | Termination => SummList (sl)
  )

  fun transform (proc) = construct (proc, SummList ([]))
pub fn transform (p : Process) -> MachineTerm {
}

*/

