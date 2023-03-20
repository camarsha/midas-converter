use crate::mdpp_bank::MDPPBank;
use crate::module_config;
use crate::write_data::WriteData;
use midasio::read::file::FileView;
use std::collections::HashMap;
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
    pub fn sort_loop<'a>(self, file_view: &'a FileView) -> HashMap<String, MDPPBank> {
        // load the configuration
        let config: module_config::Config = module_config::create_config(&self.config_file);
        // we keep a hash map of banks, this allows us to track incomplete
        // mdpp events across Midas events and hopefully complete them.
        let mut bank_hash: HashMap<_, _> = config
            .modules
            .iter()
            .map(|m| (m.name.to_string(), MDPPBank::new(&self.filename)))
            .collect();
        for (event_num, event) in (*file_view).into_iter().enumerate() {
            // select trigger events
            if event.id() == 1 {
                for bank in event {
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
            // if event_num > self.chunk_size {
            //     // only write the banks that are complete
            //     for (_key, mut value) in bank_hash.into_iter() {
            //         // start signals that a header has been read, but not an end
            //         // of event bank
            //         if !value.start {
            //             value.write_data();
            //         }
            //     }
            // }
        }
        // These are the banks that are left over if we have already dumped the data.
        bank_hash
    }
}
