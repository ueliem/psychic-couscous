#![feature(box_syntax, box_patterns)]
#[macro_use]
extern crate combine;
extern crate combine_language;
extern crate csv;

use std::fs;
use crate::combine::Parser;

mod syntax;
mod ast;
mod parser;
mod machineterm;
mod store;
mod sim;

fn main() {
    println!("Hello, world!");
    let filename = "test.spi";
    let f = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");
    let g : &str = &f;
    let prog = match parser::program().parse(combine::easy::Stream(g)) {
        Ok ((p, _)) => p,
        Err (e) => panic!("{:?}", e)
    };
    let mut sim = sim::Simulator::new();
    sim.load(&prog);
    let h : Vec<String> = sim.s.instance_counts.iter().map(|(k, v)| k.to_string()).collect();
    println!("Time, {}", h.join(", "));
    let c : Vec<String> = sim.s.instance_counts.iter().map(|(k, v)| v.to_string()).collect();
    println!("{}, {}", sim.time, c.join(", "));
    for i in 0..100000 {
        sim.reduce();
        let c1 : Vec<String> = sim.s.instance_counts.iter().map(|(k, v)| v.to_string()).collect();
        println!("{}, {}", sim.time, c1.join(", "));
    }
}

