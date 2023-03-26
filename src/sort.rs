use crate::mdpp_bank::MDPPBank;
use crate::module_config;
use crate::write_data::CSVFile;
use crate::write_data::WriteData;
use indicatif::ProgressBar;
use midasio::read::file::FileView;
use std::collections::HashMap;
use std::time::Duration;

pub struct DataSort {
    filename: String,
    chunk_size: usize,
    config_file: String,
}

impl DataSort {
    pub fn new(filename: String, chunk_size: usize, config_file: String) -> Self {
        DataSort {
            filename,
            chunk_size,
            config_file,
        }
    }

    // need to add a periodic dumping of data
    pub fn sort_loop<'a>(self, file_view: &'a FileView) {
        // load the configuration
        let config: module_config::Config = module_config::create_config(&self.config_file);
        // set up the file dumper
        let mut file_dumper = CSVFile::new(&self.filename);

        // we keep a hash map of banks, this allows us to track incomplete
        // mdpp events across Midas events and hopefully complete them.
        let mut bank_hash: HashMap<_, _> = config
            .modules
            .iter()
            .map(|m| (m.name.to_string(), MDPPBank::new()))
            .collect();
        // setup the progress bar
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(200));
        let mut events_towards_chunks: usize = 0;
        for (event_num, event) in (*file_view).into_iter().enumerate() {
            pb.tick();
            events_towards_chunks += 1;
            // junk should now be in their own banks
            // select trigger events
            if event.id() == 1 {
                for bank in event {
                    if bank.data_slice().len() == 1 {
                        continue;
                    }
                    // find the information associated with the bank name
                    let m: &module_config::Module = config
                        .modules
                        .iter()
                        .find(|&m| m.name == bank.name())
                        .unwrap();
                    // call the MDPPBank structure associated with midas bank
                    bank_hash.get_mut(&m.name).unwrap().parse(
                        &m.mod_type.to_string(),
                        m.nchannels,
                        bank.data_slice(),
                    );
                }
            }
            // write data to disk if we surpass the chunk size
            if events_towards_chunks > self.chunk_size {
                // only write the banks that are complete
                // we iterate over the config again because we don't want the
                // loop to own the hashmap.
                pb.set_message(format!("Events Processed: {}", event_num));
                events_towards_chunks = 0;

                for m in config.modules.iter() {
                    // start signals that a header has been read, but not an end
                    // of event bank
                    let temp = bank_hash.get_mut(&m.name.to_string()).unwrap();
                    file_dumper.write_data(temp);
                }
            }

            // check if we are on the last iteration
        }
        // show progress

        // These are the banks that are left over if we have already dumped the data.
        for m in config.modules.iter() {
            // start signals that a header has been read, but not an end
            // of event bank
            let temp = bank_hash.get_mut(&m.name.to_string()).unwrap();
            file_dumper.write_data(temp);
        }
    }
}
