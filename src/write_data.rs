use crate::mdpp_bank::MDPPBank;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

// This is create some amount of division between the unpacker and the writer.
pub trait WriteData {
    fn write_data(&mut self, bank_data: &mut MDPPBank) {}
}

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
}

// here is the write function for a csv.
impl WriteData for CSVFile {
    fn write_data(&mut self, bank_data: &mut MDPPBank) {
        // write the csv header if we haven't already
        if self.first_call {
            writeln!(
                self.file,
                "module,channel,adc,long,short,tdc,trigger_dt,pileup,event"
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
