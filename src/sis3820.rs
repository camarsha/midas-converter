use crate::bitmasks;

pub struct ScalerBank {
    pub data: Vec<u32>,
}

impl ScalerBank {
    pub fn new() -> Self {
        ScalerBank { data: vec![0; 32] }
    }

    pub fn parse(&mut self, bank: &[u8]) {
        for (i, chunk) in bank.chunks(4).enumerate() {
            let temp = bitmasks::to_u32_le(chunk);
            self.data[i] = temp;
        }
    }
}
