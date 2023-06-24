use crate::mdpp_bank::MDPPBank;
use crate::sis3820::ScalerBank;
use crate::v785_bank::v785Bank;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

pub struct CSVFile {
    first_call: bool,
    file: BufWriter<File>,
}

impl CSVFile {
    pub fn new(filename: &str) -> Self {
        CSVFile {
            first_call: true,
            file: BufWriter::new(File::create(filename).unwrap()),
        }
    }

    // here is the write function for a csv.
    pub fn write_data(&mut self, bank_data: &mut MDPPBank) {
        // write the csv header if we haven't already
        if self.first_call {
            writeln!(
                self.file,
                "module,channel,adc,long,short,tdc,trigger_dt,pileup,evt_ts"
            )
            .unwrap();
            self.first_call = false;
        }

        // loop through events
        for event in &bank_data.events {
            // loop through hits
            for (&chan, chan_hit) in event.channels.iter().zip(&event.channel_hits) {
                writeln!(
                    self.file,
                    "{},{},{},{},{},{},{},{},{}",
                    event.module_id,
                    chan,
                    chan_hit.adc_value,
                    chan_hit.long_value,
                    chan_hit.short_value,
                    chan_hit.tdc_value,
                    chan_hit.trigger_dt_value,
                    chan_hit.pile_up,
                    event.evt_timestamp
                )
                .unwrap();
            }
        }
        // free the memory for the old events
        bank_data.clear_data();
    }
}

pub struct CSVScaler {
    first_call: bool,
    file: BufWriter<File>,
}

impl CSVScaler {
    pub fn new(filename: &str) -> Self {
        CSVScaler {
            first_call: true,
            file: BufWriter::new(File::create(filename).unwrap()),
        }
    }

    fn write_header(&mut self) {
        // write the csv header if we haven't already
        let chan_str = (0..32)
            .map(|i| format!("chan_{}", i))
            .collect::<Vec<String>>()
            .join(",");
        writeln!(self.file, "{}", chan_str).unwrap();
        self.first_call = false;
    }

    pub fn write_data(&mut self, bank_data: &mut ScalerBank) {
        if self.first_call {
            self.write_header();
        }

        // loop through scaler data
        writeln!(
            self.file,
            "{}",
            bank_data
                .data
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
        .unwrap();
    }
}

/// Structs for the v785

pub struct CSVv785 {
    first_call: bool,
    file: BufWriter<File>,
}

impl CSVv785 {
    pub fn new(filename: &str) -> CSVv785 {
        CSVv785 {
            first_call: true,
            file: BufWriter::new(File::create(filename).unwrap()),
        }
    }

    fn write_header(&mut self) {
        // write the csv header
        writeln!(self.file, "module,channel,adc,evt_ts").unwrap();
        self.first_call = false;
    }

    pub fn write_data(&mut self, bank_data: &mut v785Bank) {
        // check if we need to write the header
        if self.first_call {
            self.write_header();
        }

        // the v785 data is a bit different and is already stored as a vector
        // loop through the hits
        for hit in bank_data.hits.iter() {
            for i in 0..32 {
                //                let value_str = hit.values[i as usize].to_string();
                writeln!(
                    self.file,
                    "{},{},{},{}",
                    0, i, hit.values[i as usize], hit.evt
                )
                .unwrap();
            }
        }
    }
}
