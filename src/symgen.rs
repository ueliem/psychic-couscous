use std::sync::Mutex;

#[derive(Debug)]
pub struct SymbolGenerator {
    counter : usize
}

lazy_static! {
    pub static ref SYMGEN : Mutex<SymbolGenerator> = Mutex::new(SymbolGenerator {
        counter : 0
    });
}

pub fn next() -> String {
    let n = SYMGEN.lock().unwrap().counter;
    SYMGEN.lock().unwrap().counter += 1;
    return n.to_string();
}

pub fn reset() {
    SYMGEN.lock().unwrap().counter = 0;
}

