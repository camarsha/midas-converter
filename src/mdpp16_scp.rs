use crate::bitmasks;
use crate::WriteData;
use std::fs::File;
use std::io::Write;
/* General Outline of how this works:

We have a nested struct which Hit -> Event -> Bank

The bank struct does all of the bit shifting/masking and sends that data to the
current event. The event struct adds that data to the header, adc, tdc, or end event. With in the current event all of the hits are grouped such that the adc and tdc values form the same channel are grouped.
*/

// I want a way to match adc and tdc values so that they can later
// be printed out to a single row in the output file.
pub struct MDPPHit {
    pub adc_value: u32,
    pub tdc_value: u32,
    pub pile_up: bool,
    pub overflow: bool,
    tdc_filled: bool,
    adc_filled: bool,
}

impl MDPPHit {
    pub fn new() -> Self {
        MDPPHit {
            adc_value: 0,
            adc_filled: false,
            tdc_value: 0,
            tdc_filled: false,
            pile_up: false,
            overflow: false,
        }
    }

    pub fn set_adc(&mut self, adc_value: u32, pile_up: bool, overflow: bool) -> bool {
        let mut already_filled = false;
        if !self.adc_filled {
            self.adc_value = adc_value;
            self.pile_up = pile_up;
            self.overflow = overflow;
            self.adc_filled = true;
        } else {
            already_filled = true;
        }
        already_filled
    }

    pub fn set_tdc(&mut self, tdc_value: u32) -> bool {
        let mut already_filled = false;
        if !self.tdc_filled {
            self.tdc_value = tdc_value;
            self.tdc_filled = true;
        } else {
            already_filled = true;
        }
        already_filled
    }
}

// An event is made up of hits. This handles the logic about what channels have already fired.
pub struct MDPPEvent {
    pub module_id: u32,
    nwords: u32,
    pub event_num: u32,
    pub channels: Vec<u32>,
    pub channel_hits: Vec<MDPPHit>,
}

impl MDPPEvent {
    pub fn new(module_id: u32, nwords: u32) -> Self {
        MDPPEvent {
            module_id,
            nwords,
            event_num: 0,
            channels: Vec::with_capacity(16),
            channel_hits: Vec::with_capacity(16),
        }
    }

    pub fn add_adc(&mut self, channel: u32, adc_value: u32, pile_up: bool, overflow: bool) {
        let mut channel_found = false;
        // branch that handles if the channel is already fired
        for (i, &c) in self.channels.iter().enumerate() {
            if channel == c {
                self.channel_hits[i].set_adc(adc_value, pile_up, overflow);
                channel_found = true;
            }
        }
        if !channel_found {
            self.channels.push(channel);
            self.channel_hits.push(MDPPHit::new());
            let current_len = self.channel_hits.len() - 1;
            self.channel_hits[current_len].set_adc(adc_value, pile_up, overflow);
        }
    }

    pub fn add_tdc(&mut self, channel: u32, tdc_value: u32) {
        let mut channel_found = false;
        // branch that handles if the channel is already fired
        for (i, &c) in self.channels.iter().enumerate() {
            if channel == c {
                self.channel_hits[i].set_tdc(tdc_value);
                channel_found = true;
            }
        }
        if !channel_found {
            self.channels.push(channel);
            self.channel_hits.push(MDPPHit::new());
            let current_len = self.channel_hits.len() - 1;
            self.channel_hits[current_len].set_tdc(tdc_value);
        }
    }
    pub fn end_event(&mut self, event_num: u32) {
        self.event_num = event_num;
    }
}

// This handles the bank data which is made up of a present number of events thanks to chunk size.
// It also handles unpacking the binary data.

pub struct MDPPBank {
    pub events: Vec<MDPPEvent>,
    current_event: usize,
    chunk_size: usize,
    file_created: bool,
}

// see manual for a break down
impl MDPPBank {
    // create a new MDPP bank object that initialize with a chunk size.
    pub fn new(chunk_size: usize) -> Self {
        MDPPBank {
            events: Vec::with_capacity(chunk_size),
            current_event: 0,
            // two variables that handle the file dumping.
            chunk_size,
            file_created: false,
        }
    }

    pub fn parse(&mut self, bank: &[u8]) {
        // start looping through the data 32 bit words
        for chunk in bank.chunks(4) {
            let temp = bitmasks::to_u32_le(chunk);
            // get the identifier
            let data_sig = temp >> 30 & bitmasks::TWO_BIT;
            match data_sig {
                0 => Self::parse_data(self, temp),
                1 => Self::parse_header(self, temp),
                3 => Self::parse_end_event(self, temp),
                _ => panic!("Invalid Data signature in bank!"),
            }
        }
    }

    fn parse_header(&mut self, header: u32) {
        // here we create the event
        let module_id: u32 = header >> 16 & bitmasks::EIGHT_BIT;
        let nwords: u32 = bitmasks::TEN_BIT;
        self.events.push(MDPPEvent::new(module_id, nwords));
    }

    fn parse_end_event(&mut self, end_event: u32) {
        let event_num = end_event & bitmasks::THIRTY_BIT;
        self.events[self.current_event].end_event(event_num);
        self.current_event += 1;
    }

    fn parse_data(&mut self, data: u32) {
        // Check if this event has data or is just filler
        let subheader = data >> 28 & bitmasks::THREE_BIT;
        match subheader {
            0 => (),                               // dummy event
            1 => Self::push_data(self, data),      // actual data
            10 => println!("Extended TimeStmap",), // extended timestamp
            _ => panic!("Invalid subheader in bank word!"),
        }
    }

    fn push_data(&mut self, data: u32) {
        let mut channel = data >> 16 & bitmasks::FIVE_BIT;
        // tdc event
        if channel > 15 {
            channel -= 16;
            let tdc = data & bitmasks::SIXTEEN_BIT;
            self.events[self.current_event].add_tdc(channel, tdc);
        // adc event
        } else {
            let adc = data & bitmasks::SIXTEEN_BIT;
            let pile_up = (data >> 23 & bitmasks::ONE_BIT) != 0;
            let overflow = (data >> 22 & bitmasks::ONE_BIT) != 0;

            self.events[self.current_event].add_adc(channel, adc, pile_up, overflow);
        }
    }

    pub fn clear_data(&mut self) {
        self.current_event = 0;
        self.events = Vec::with_capacity(self.chunk_size);
    }
}

// here is the write function for a csv.
impl WriteData for MDPPBank {
    fn write_data(&mut self, filename: &str) {
        // open the file create it if necessary
        let file_option = if !self.file_created {
            File::create(filename)
        } else {
            File::options().append(true).open(filename)
        };
        // unpack the option
        let mut f = if let Ok(i) = file_option {
            i
        } else {
            panic!("Issue creating output file!")
        };

        // write the csv header if we haven't already
        if !self.file_created {
            writeln!(f, "module,channel,adc,tdc,pileup,overlow,event");
            self.file_created = true;
        }

        // loop through events
        for event in &self.events {
            // loop through hits
            for (&chan, chan_hit) in event.channels.iter().zip(&event.channel_hits) {
                writeln!(
                    f,
                    "{},{},{},{},{},{},{}",
                    event.module_id,
                    chan,
                    chan_hit.adc_value,
                    chan_hit.tdc_value,
                    chan_hit.pile_up as u8,
                    chan_hit.overflow as u8,
                    event.event_num
                );
            }
        }
        // free the memory for the old events
        self.clear_data();
    }
}
