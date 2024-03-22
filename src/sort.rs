use crate::mdpp_bank::MDPPBank;
use crate::module_config;
use crate::sis3820::ScalerBank;
use crate::v785_bank::v785Bank;
use crate::write_data::{CSVFile, CSVScaler, CSVv785};
use indicatif::ProgressBar;
use midasio::read::file::FileView;
use std::collections::HashMap;
use std::io::{stdin, stdout, Write};
use std::time::Duration;

fn strip_trailing_newline(input: &str) -> &str {
    input
        .strip_suffix("\r\n")
        .or(input.strip_suffix('\n'))
        .unwrap_or(input)
}

pub struct MDPPSort {
    filename: String,
    scaler_filename: String,
    chunk_size: usize,
    config: module_config::Config,
}

impl MDPPSort {
    pub fn new(
        filename: String,
        scaler_filename: String,
        chunk_size: usize,
        config: module_config::Config,
    ) -> Self {
        MDPPSort {
            filename,
            scaler_filename,
            chunk_size,
            config,
        }
    }

    pub fn sort_loop<'a>(self, file_view: &'a FileView) {
        // set up the file dumper
        let mut mdpp_file_dumper = CSVFile::new(&self.filename);
        let mut scaler_file_dumper = CSVScaler::new(&self.scaler_filename);

        // we keep a hash map of banks, this allows us to track incomplete
        // mdpp events across Midas events and hopefully complete them.
        let mut bank_hash: HashMap<_, _> = self
            .config
            .modules
            .iter()
            .map(|m| (m.name.to_string(), MDPPBank::new()))
            .collect();
        // the scaler banks are simple, and do not require much abstraction
        let mut scaler_banks: Vec<ScalerBank> = vec![ScalerBank::new()];
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
                    let m: Option<&module_config::Module> =
                        self.config.modules.iter().find(|&m| m.name == bank.name());
                    // call the MDPPBank structure associated with midas bank, if the bank name is invalid let the user know.
                    match m {
                        Some(m) => bank_hash.get_mut(&m.name).unwrap().parse(
                            &m.mod_type.to_string(),
                            m.nchannels,
                            bank.data_slice(),
                        ),
                        None => {
                            println!("No bank matching name {}", bank.name());
                            println!("Press enter to continue, enter any other string to quit.");
                            let mut user_choice = String::new();
                            let _ = stdout().flush();
                            stdin()
                                .read_line(&mut user_choice)
                                .expect("A valid string was not entered");
                            if strip_trailing_newline(&user_choice).is_empty() {
                                continue;
                            } else {
                                panic! {"User has aborted reading of file after invalid bank name."};
                            }
                        }
                    }
                }
            } else if event.id() == 2 {
                for bank in event {
                    let mut temp = ScalerBank::new();
                    temp.parse(bank.data_slice());
                    scaler_banks.push(temp);
                }
            }

            // write data to disk if we surpass the chunk size
            if events_towards_chunks > self.chunk_size {
                // only write the banks that are complete
                // we iterate over the config again because we don't want the
                // loop to own the hashmap.
                pb.set_message(format!("Events Processed: {}", event_num));
                events_towards_chunks = 0;

                for m in self.config.modules.iter() {
                    // start signals that a header has been read, but not an end
                    // of event bank
                    let temp = bank_hash.get_mut(&m.name.to_string()).unwrap();
                    mdpp_file_dumper.write_data(temp);
                }
            }

            // check if we are on the last iteration
        }
        // show progress

        // These are the banks that are left over if we have already dumped the data.
        for m in self.config.modules.iter() {
            // start signals that a header has been read, but not an end
            // of event bank
            let temp = bank_hash.get_mut(&m.name.to_string()).unwrap();
            mdpp_file_dumper.write_data(temp);
        }
        // dump the scalers to their own csv file.
        for i in 0..scaler_banks.len() {
            scaler_file_dumper.write_data(&mut scaler_banks[i]);
        }
    }
}

// This is just copy and paste for now. Too lazy to try and abstract away the similarities

pub struct v785Sort {
    filename: String,
    scaler_filename: String,
    chunk_size: usize,
    config: module_config::Config,
}

impl v785Sort {
    pub fn new(
        filename: String,
        scaler_filename: String,
        chunk_size: usize,
        config: module_config::Config,
    ) -> v785Sort {
        v785Sort {
            filename,
            scaler_filename,
            chunk_size,
            config,
        }
    }

    pub fn sort_loop<'a>(self, file_view: &'a FileView) {
        // set up the file dumper
        let mut v785_file_dumper = CSVv785::new(&self.filename);
        let mut scaler_file_dumper = CSVScaler::new(&self.scaler_filename);

        // we keep a hash map of banks, this allows us to track incomplete
        // mdpp events across Midas events and hopefully complete them.
        let mut bank_hash: HashMap<_, _> = self
            .config
            .modules
            .iter()
            .map(|m| (m.name.to_string(), v785Bank::new()))
            .collect();
        // the scaler banks are simple, and do not require much abstraction
        let mut scaler_banks: Vec<ScalerBank> = vec![ScalerBank::new()];
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
                    let m: Option<&module_config::Module> =
                        self.config.modules.iter().find(|&m| m.name == bank.name());
                    match m {
                        Some(m) => bank_hash.get_mut(&m.name).unwrap().parse(bank.data_slice()),
                        None => continue,
                    }
                    // Most of this is pointless right now, the data should be simple
                }
            } else if event.id() == 2 {
                for bank in event {
                    let mut temp = ScalerBank::new();
                    temp.parse(bank.data_slice());
                    scaler_banks.push(temp);
                }
            }

            // write data to disk if we surpass the chunk size
            if events_towards_chunks > self.chunk_size {
                // only write the banks that are complete
                // we iterate over the config again because we don't want the
                // loop to own the hashmap.
                pb.set_message(format!("Events Processed: {}", event_num));
                events_towards_chunks = 0;

                for m in self.config.modules.iter() {
                    // start signals that a header has been read, but not an end
                    // of event bank
                    let temp = bank_hash.get_mut(&m.name.to_string()).unwrap();
                    v785_file_dumper.write_data(temp);
                }
            }

            // check if we are on the last iteration
        }
        // show progress

        // These are the banks that are left over if we have already dumped the data.
        for m in self.config.modules.iter() {
            // start signals that a header has been read, but not an end
            // of event bank
            let temp = bank_hash.get_mut(&m.name.to_string()).unwrap();
            v785_file_dumper.write_data(temp);
        }
        // dump the scalers to their own csv file.
        for i in 0..scaler_banks.len() {
            scaler_file_dumper.write_data(&mut scaler_banks[i]);
        }
    }
}
