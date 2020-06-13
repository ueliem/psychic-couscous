use std::collections::BTreeMap;
use std::rc::Rc;

use super::values::*;
use super::ast;

#[derive(Debug)]
pub struct ChannelRecord {
    pub rate : f64, 
    pub incount : usize, 
    pub outcount : usize, 
    pub mixcount : usize, 
    pub ax : f64
}

#[derive(Debug)]
pub struct Store {
    pub chans: BTreeMap<String, ChannelRecord>,
    pub defs: BTreeMap<String, (Vec<Pattern>, Rc<ast::Process>)>,
    pub instance_counts: BTreeMap<String, usize>
}

impl Store {
    pub fn new() -> Store {
        Store {chans : BTreeMap::new(), defs : BTreeMap::new(), instance_counts : BTreeMap::new()}
    }
    pub fn add_channel(&mut self, name : &str, rate : f64) {
        self.chans.insert(name.to_string(),
            ChannelRecord {
                rate: rate,
                incount : 0,
                outcount : 0,
                mixcount : 0,
                ax : 0.0
            });
    }
    pub fn activities(&self) -> Vec<(String, f64)> {
        fn activity ((k, c) : (&str, &ChannelRecord)) -> Option<(String, f64)> {
            if c.ax > 0.0 {
                Some ((k.to_string(), c.ax as f64 * c.rate))
            }
            else {
                None
            }
        }
        self.chans.iter().filter_map(|(k, c)| activity((k, c))).collect()
    }
    pub fn add_counts(&mut self, counts : BTreeMap<&str, (usize, usize, usize)>) {
        for (k, v) in counts.iter() {
            self.chans.entry(k.to_string()).and_modify(|c| {
                c.incount += v.0;
                c.outcount += v.1;
                c.mixcount += v.2;
                c.ax = ((c.incount * c.outcount) - c.mixcount) as f64;
            });
        }
    }
    pub fn remove_counts(&mut self, counts : BTreeMap<&str, (usize, usize, usize)>) {
        for (k, v) in counts.iter() {
            self.chans.entry(k.to_string()).and_modify(|c| {
                c.incount -= v.0;
                c.outcount -= v.1;
                c.mixcount -= v.2;
                c.ax = ((c.incount * c.outcount) - c.mixcount) as f64;
            });
        }
    }
    pub fn create(&mut self, instance_name : String) {
        *self.instance_counts.entry(instance_name).or_insert(0) += 1;
    }
    pub fn destroy(&mut self, instance_name : String) {
        *self.instance_counts.entry(instance_name).or_insert(0) -= 1;
    }
}

