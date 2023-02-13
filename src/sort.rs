use crate::bitmasks;
use crate::mdpp16_scp;
use crate::mdpp16_scp::MDPPBank;
use crate::write_data::WriteData;
use itertools::Itertools;
use midasio::read::data_bank::Bank32View;
use midasio::read::file::FileView;
use midasio::DataType;
use std::error::Error;
use std::path::Path;

pub struct DataSort {
    filename: String,
    chunk_size: usize,
}

impl DataSort {
    pub fn new(filename: String, chunk_size: usize) -> Self {
        DataSort {
            filename,
            chunk_size,
        }
    }

    // need to add a periodic dumping of data
    pub fn sort_loop<'a>(self, file_view: &'a FileView) -> MDPPBank {
        let mut banks = MDPPBank::new(self.chunk_size);
        let mut banks_proccessed: usize = 0;
        for event in (*file_view).into_iter() {
            // select physics events
            if event.id() == 1 {
                for bank in event {
                    banks_proccessed += 1;
                    if bank.name() == "MDPP" {
                        banks.parse(bank.data_slice());
                    }
                }
            }
            // write data to disk if we surpass the chunk size
            if banks_proccessed > self.chunk_size {
                banks.write_data(&self.filename);
            }
        }
        banks
    }
}
