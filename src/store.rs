use std::collections::HashMap;

pub type ChannelRecord = (f64, usize, usize, usize, f64);

#[derive(Debug)]
pub struct Store {
    chans: HashMap<String, ChannelRecord>
}

impl Store {
    pub fn new() -> Store {
        Store {chans : HashMap::new()}
    }
    pub fn add_channel(&mut self, name : String) {
        self.chans.insert(name, (0.0, 0, 0, 0, 0.0));
    }
}

