#![feature(box_syntax, box_patterns)]
#[macro_use]
extern crate combine;
extern crate combine_language;

use std::rc::Rc;
use crate::combine::Parser;

mod syntax;
mod ast;
mod parser;
mod machineterm;
mod store;
mod sim;


fn main() {
    println!("Hello, world!");
    let prog = match parser::program().easy_parse("new CHAN@1.0\nrun a?b{(end | end | end)}") {
        Ok ((p, _)) => p,
        Err (e) => panic!("{}", e)
    };
    let mut sim = sim::Simulator::new();
    sim.load(&prog);
    println!("{:?}", sim);
    sim.reduce();
    parser::myparser();
    println!("{:?}", sim);
}

