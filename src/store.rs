use std::collections::HashMap;

#[derive(Debug)]
pub struct ChannelRecord {
    rate : f64, 
    incount : usize, 
    outcount : usize, 
    mixcount : usize, 
    ax : f64
}

#[derive(Debug)]
pub struct Store {
    chans: HashMap<String, ChannelRecord>
}

impl Store {
    pub fn new() -> Store {
        Store {chans : HashMap::new()}
    }
    pub fn add_channel(&mut self, name : &String, rate : f64) {
        self.chans.insert(name.clone(),
            ChannelRecord {
                rate: rate,
                incount : 0,
                outcount : 0,
                mixcount : 0,
                ax : 0.0
            });
    }
    pub fn activities(&self) -> Vec<(String, f64)> {
        fn activity ((k, c) : (&String, &ChannelRecord)) -> Option<(String, f64)> {
            let a = c.incount * c.outcount - c.mixcount;
            if a > 0 {
                Some ((k.clone(), a as f64 * c.rate))
            }
            else {
                None
            }
        }
        self.chans.iter().filter_map(
        // |(k, c)| Some ((k, c.incount * c.outcount - c.mixcount))).collect()
        activity).collect()
    }
}

