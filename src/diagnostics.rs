use crate::bitmasks;
use midasio::read::file::FileView;

/*
This module can be used to debug issues with the Midas frontend.
Right now it just provides functions to tally the number of event
headers and events ends.
*/

pub fn event_diagnostics<'a>(file_view: &'a FileView) -> (u32, u32, u32) {
    let mut total_banks: u32 = 0;
    let mut total_headers: u32 = 0;
    let mut total_event_ends: u32 = 0;
    for (_event_num, event) in (*file_view).into_iter().enumerate() {
        if event.id() == 1 {
            for bank in event {
                total_banks += 1;
                for chunk in bank.data_slice().chunks(4) {
                    let temp = bitmasks::to_u32_le(chunk);
                    // get the identifier
                    let data_sig = temp >> 30 & bitmasks::TWO_BIT;
                    match data_sig {
                        0 => {}
                        1 => total_headers += 1,
                        3 => total_event_ends += 1,
                        _ => panic!("Invalid Data signature in bank!"),
                    }
                }
            }
        }
    }

    (total_banks, total_headers, total_event_ends)
}
