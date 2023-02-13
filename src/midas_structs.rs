use binread::{io::Cursor, BinRead};
use std::io::BufRead;
use std::io::BufReader;

#[derive(Debug, BinRead)]
pub struct EventHeader {
    event_id: u16,
    trigger_mask: u16,
    serial_number: u32,
    time_stamp: u32,
    event_size: u32,
}

#[derive(Debug, BinRead)]
pub struct BankHeader {
    all_banks_size: u32,
    flags: u32,
}

#[derive(Debug, BinRead)]
pub struct Byte {
    all_banks_size: u8,
}
