#![feature(box_syntax, box_patterns)]
#![recursion_limit = "87"]
#[macro_use]
extern crate combine;
extern crate combine_language;
extern crate csv;
extern crate diff_enum;
#[macro_use]
extern crate lazy_static;

use std::fs;
use combine::Parser;
use structopt::StructOpt;

mod symgen;
mod tokenizer;
mod values;
mod lambda;
mod syntax;
mod ast;
mod parser;
mod machineterm;
mod store;
mod sim;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    inpath: std::path::PathBuf,
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    outpath: std::path::PathBuf,
}

fn main() {
    let args = Cli::from_args();
    let filename = args.inpath; // "test.spi";
    let f = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");
    let g : &str = &f;
    let mut s = combine::easy::Stream(g);
    let p = tokenizer::tokenize().parse_stream(&mut s);
    println!("{:?}", p);
    let z = parser::program().parse_stream(&mut (p.unwrap().0.as_slice()));
    println!("{:?}", z);
    let prog = match z {
        Ok ((p, c)) => {
            println!("{:?} {:?}", p, c);
            p
        },
        Err (e) => panic!("{:?}", e)
    };
    let mut sim = sim::Simulator::new();
    sim.load(&prog);
    let mut wtr = csv::Writer::from_path(args.outpath).unwrap();
    let mut headers : Vec<String> = sim.s.instance_counts.iter().map(|(k, _v)| k.to_string()).collect();
    headers.insert(0, "Time".to_string());
    wtr.write_record(headers).unwrap();
    let mut c : Vec<String> = sim.s.instance_counts.iter().map(|(_k, v)| v.to_string()).collect();
    c.insert(0, sim.time.to_string());
    wtr.write_record(c).unwrap();
    for _i in 0..1000000 {
        sim.reduce();
        let mut c1 : Vec<String> = sim.s.instance_counts.iter().map(|(_k, v)| v.to_string()).collect();
        c1.insert(0, sim.time.to_string());
        wtr.write_record(c1).unwrap();
    }
}

