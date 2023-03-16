use crate::mdpp_bank::MDPPBank;
use crate::module_config;
use crate::write_data::WriteData;
use midasio::read::file::FileView;

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
    pub fn sort_loop<'a>(self, file_view: &'a FileView) -> MDPPBank {
        // load the configuration
        let config: module_config::Config = module_config::create_config(&self.config_file);
        let mut banks = MDPPBank::new(&self.filename);
        for (event_num, event) in (*file_view).into_iter().enumerate() {
            // select trigger events
            if event.id() == 1 {
                for bank in event {
                    let m: &module_config::Module = config
                        .modules
                        .iter()
                        .find(|&m| m.name == bank.name())
                        .unwrap();

                    banks.parse(&m.mod_type.to_string(), m.nchannels, bank.data_slice());
                }
            }
            // write data to disk if we surpass the chunk size
            if event_num > self.chunk_size {
                banks.write_data();
            }
        }
        // These are the banks that are left over if we have already dumped the data.
        banks
    }
}
