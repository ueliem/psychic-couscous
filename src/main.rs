#![feature(box_syntax, box_patterns)]

mod ast;
mod machineterm;
mod store;
mod sim;

fn main() {
    println!("Hello, world!");
    let prog = ast::Process::Restriction ("a".to_string(), box ast::Process::Parallel (box ast::Process::Termination, box ast::Process::Termination));
    let mut sim = sim::Simulator::new();
    sim.load(&prog);
    println!("{:?}", sim);
}
