use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::i64;
use toml;

#[derive(Debug, Deserialize, Clone)]
pub struct Module {
    pub name: String,
    address: String,
    pub mod_type: String,
    pub nchannels: u32,
    setup_file: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub modules: Vec<Module>,
}

pub fn create_config(config_filename: &str) -> Config {
    let contents = match fs::read_to_string(config_filename) {
        Ok(c) => c,
        Err(_) => {
            panic!();
        }
    };

    let data: Config = toml::from_str(&contents).unwrap();
    data
}
