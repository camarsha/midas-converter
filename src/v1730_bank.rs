use std::default;

use itertools::Itertools;

use crate::bitmasks;

/*
For right now I am injesting data from the front end written
by W. Fox which strips out all of the useless header info.

 */
#[derive(Default)]
pub struct v1730Hit {
    pub channel: u32,
    pub long: u32,
    pub coarse_time: u64,
    pub time: f64,
}

impl v1730Hit {
    pub fn new(hit_data: &[u32]) -> Self {
        // I took all of these without checking if they work correctly.

        let channel = (hit_data[0] >> 16) & bitmasks::FOUR_BIT;
        let long = hit_data[0] & bitmasks::SIXTEEN_BIT;
        let time_std = hit_data[1] as u64;
        let time_ext = ((hit_data[2] >> 16) & bitmasks::SIXTEEN_BIT) as u64;
        let coarse_time = (time_ext << 31) + time_std;
        let fine_time = hit_data[2] & bitmasks::TEN_BIT;
        let time = (coarse_time as f64) + ((fine_time as f64) / 1024.0);

        v1730Hit {
            channel,
            long,
            coarse_time,
            time,
        }
    }
}

pub struct v1730Bank {
    pub hits: Vec<v1730Hit>,
}

impl v1730Bank {
    pub fn new() -> Self {
        v1730Bank {
            hits: Vec::with_capacity(100),
        }
    }

    pub fn parse(&mut self, bank: &[u8]) {
        // Each bank consists of 32 bit words
        // in groups of 3 for the channel data.
        // We need to combine the u8 to u32 then group again into .
        let all_words: Vec<u32> = bank.chunks(4).map(bitmasks::to_u32_le).collect();
        all_words
            .chunks(3)
            .for_each(|hit| self.hits.push(v1730Hit::new(hit)));
    }
}
