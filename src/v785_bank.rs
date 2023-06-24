use crate::bitmasks;

/*
This the for the v785 ADC bank information.

It might not require too much fiddling. The front end is already
doing the heavy lifting.

The only wrinkle is that we have to keep track of the event numbers ourselves.

 */

#[derive(Default)]
pub struct v785Hit {
    pub values: [u32; 32],
    pub evt: u32,
}

impl v785Hit {
    pub fn new(evt: u32) -> v785Hit {
        let values: [u32; 32] = [0; 32];
        v785Hit { values, evt }
    }
}

pub struct v785Bank {
    pub hits: Vec<v785Hit>,
    global_evt_num: u32,
}

impl v785Bank {
    pub fn new() -> v785Bank {
        v785Bank {
            hits: Vec::with_capacity(100),
            global_evt_num: 0,
        }
    }

    fn add_hit(&mut self, data: &[u8]) {
        let mut current_hit = v785Hit::new(self.global_evt_num);
        // convert the 4 u8 numbers to a single u32 number
        let all_data: Vec<u32> = data.chunks(4).map(bitmasks::to_u32_le).collect();
        // there are 34 u32 numbers in the bank, we only care about the first 32
        for i in 0..32 {
            current_hit.values[i as usize] = all_data[i as usize];
        }
        current_hit.evt = self.global_evt_num;
        // store and increment
        self.global_evt_num += 1;
        self.hits.push(current_hit);
    }

    pub fn parse(&mut self, bank: &[u8]) {
        self.add_hit(bank);
    }
}
