use crate::bitmasks;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

/* This is going to be a refactoring of the ideas present in the
original mdpp16_scp.rs file.

So, aligning with the way I did this with PXI cards, we just account for all
possible configurations in a row. That means, qdc, scp, who gives a shit we just
fill in the values if they are there and leave sauce to do the rest.

*/

/*
---------- Hit Structure ----------
*/

// These are the values that can differ between hits in an event.
// Also these will not change based on the total number of channels.
#[derive(Default)]
pub struct MDPPHit {
    pub adc_value: u32,
    pub long_value: u32,
    pub short_value: u32,
    pub tdc_value: u32,
    pub trigger_dt_value: i64,
    pub pile_up: bool,
    adc_filled: bool,
    long_filled: bool,
    short_filled: bool,
    tdc_filled: bool,
    trigger_dt_filled: bool,
}

impl MDPPHit {
    // Just initialize, the trigger_dt_starting value will have to be reexamined if
    // we ever use it.
    pub fn new() -> Self {
        MDPPHit {
            ..Default::default()
        }
    }

    pub fn set_adc(&mut self, adc_value: u32, pile_up: bool) -> bool {
        let mut already_filled = false;
        if !self.adc_filled {
            self.adc_value = adc_value;
            self.pile_up = pile_up;
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

    pub fn set_long(&mut self, long_value: u32) -> bool {
        let mut already_filled = false;
        if !self.long_filled {
            self.long_value = long_value;
            self.long_filled = true;
        } else {
            already_filled = true;
        }
        already_filled
    }

    pub fn set_short(&mut self, short_value: u32) -> bool {
        let mut already_filled = false;
        if !self.short_filled {
            self.short_value = short_value;
            self.short_filled = true;
        } else {
            already_filled = true;
        }
        already_filled
    }

    pub fn set_trigger_dt(&mut self, trigger_dt: u32) -> bool {
        let mut already_filled = false;
        if !self.trigger_dt_filled {
            self.short_value = trigger_dt;
            self.trigger_dt_filled = true;
        } else {
            already_filled = true;
        }
        already_filled
    }
}

/*
---------- Event Structure ----------
 */

// This holds the information that will remain the same for hits
// within the same MDPP event, which should be a single Midas bank.
pub struct MDPPEvent {
    pub module_id: u32,
    pub evt_timestamp: u64, // depends on setup either event counter or timestamp
    pub channels: Vec<u32>,
    pub channel_hits: Vec<MDPPHit>,
    extended_ts: u32,
    extended_ts_filled: bool,
}

impl MDPPEvent {
    pub fn new(module_id: u32) -> Self {
        MDPPEvent {
            module_id,
            evt_timestamp: 0,
            channels: Vec::with_capacity(32),
            channel_hits: Vec::with_capacity(32),
            extended_ts: 0,
            extended_ts_filled: false,
        }
    }

    /*
    All of the these set functions do roughly the same thing. If a channel has been
    found, then start grouping the additional information into the MDPPHit struct. Else
    we push a new hit.
    */

    // set the adc value for the hit
    pub fn add_adc(&mut self, channel: u32, adc_value: u32, pile_up: bool) {
        let mut channel_found = false;
        // branch that handles if the channel is already fired
        for (i, &c) in self.channels.iter().enumerate() {
            if channel == c {
                self.channel_hits[i].set_adc(adc_value, pile_up);
                channel_found = true;
            }
        }
        if !channel_found {
            self.channels.push(channel);
            self.channel_hits.push(MDPPHit::new());
            let current_len = self.channel_hits.len() - 1;
            self.channel_hits[current_len].set_adc(adc_value, pile_up);
        }
    }

    // set the tdc time for the hit
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

    // set the long integral for the hit
    pub fn add_long(&mut self, channel: u32, long_value: u32) {
        let mut channel_found = false;
        // branch that handles if the channel is already fired
        for (i, &c) in self.channels.iter().enumerate() {
            if channel == c {
                self.channel_hits[i].set_long(long_value);
                channel_found = true;
            }
        }
        if !channel_found {
            self.channels.push(channel);
            self.channel_hits.push(MDPPHit::new());
            let current_len = self.channel_hits.len() - 1;
            self.channel_hits[current_len].set_long(long_value);
        }
    }

    // set the short integral for the hit
    pub fn add_short(&mut self, channel: u32, short_value: u32) {
        let mut channel_found = false;
        // branch that handles if the channel is already fired
        for (i, &c) in self.channels.iter().enumerate() {
            if channel == c {
                self.channel_hits[i].set_short(short_value);
                channel_found = true;
            }
        }
        if !channel_found {
            self.channels.push(channel);
            self.channel_hits.push(MDPPHit::new());
            let current_len = self.channel_hits.len() - 1;
            self.channel_hits[current_len].set_short(short_value);
        }
    }

    // Set the event counter or timestamp
    pub fn end_event(&mut self, evt_timestamp: u32) {
        // go ahead and adjust the timestamp if it has been extended
        if self.extended_ts_filled {
            // 16 high bits are in the extended timestamp
            self.evt_timestamp = ((self.extended_ts as u64) * 2u64.pow(30)) + evt_timestamp as u64;
        } else {
            self.evt_timestamp = evt_timestamp as u64;
        }
    }

    // Set the extended timestamp
    pub fn extended_ts(&mut self, timestamp: u32) {
        self.extended_ts = timestamp;
        self.extended_ts_filled = true;
    }

    // Set the event counter or timestamp
    pub fn add_trigger_dt(&mut self, channel: u32, trigger_dt_value: u32) {
        let mut channel_found = false;
        // branch that handles if the channel is already fired
        for (i, &c) in self.channels.iter().enumerate() {
            if channel == c {
                self.channel_hits[i].set_trigger_dt(trigger_dt_value);
                channel_found = true;
            }
        }
        if !channel_found {
            self.channels.push(channel);
            self.channel_hits.push(MDPPHit::new());
            let current_len = self.channel_hits.len() - 1;
            self.channel_hits[current_len].set_short(trigger_dt_value);
        }
    }
}

/*
---------- Bank Structure ----------
 */

// We finally have the part of the implementation
// that will actually deal with the differences between scp/qdc and the
// number of channels in a module.

pub struct MDPPBank {
    pub events: Vec<MDPPEvent>,
    current_event: usize,
    pub start: bool,
    pub stop: bool,
    pub junk_events: i64,
}
/*

*/

impl MDPPBank {
    // create a new MDPP bank object that initialize with a chunk size.
    pub fn new() -> Self {
        MDPPBank {
            events: Vec::with_capacity(10000000),
            current_event: 0,
            start: false,
            stop: false,
            junk_events: 0,
        }
    }

    pub fn parse(&mut self, bank_type: &str, nchannels: u32, bank: &[u8]) {
        // start looping through the data 32 bit words
        for chunk in bank.chunks(4) {
            let temp = bitmasks::to_u32_le(chunk);
            // get the identifier
            let data_sig = temp >> 30 & bitmasks::TWO_BIT;
            match data_sig {
                // match again based on the type of module it is
                0 => match bank_type {
                    "qdc" => Self::parse_qdc(self, nchannels, temp),
                    "scp" => Self::parse_scp(self, nchannels, temp),
                    _ => panic!("Unknown module type: {}", bank_type),
                },
                1 => Self::parse_header(self, temp),
                3 => Self::parse_end_event(self, temp),
                _ => panic!("Invalid Data signature in bank!"),
            }
        }
    }

    fn parse_header(&mut self, header: u32) {
        // here we create the event
        let module_id: u32 = header >> 16 & bitmasks::EIGHT_BIT;
        let _nwords: u32 = bitmasks::TEN_BIT;
        if self.start {
            // this only happens if the last event is ill formed, so get rid of it.
            self.events.pop();
            self.junk_events += 1;
            return;
        }
        self.start = true;
        self.events.push(MDPPEvent::new(module_id));
    }

    fn parse_end_event(&mut self, end_event: u32) {
        let event_num = end_event & bitmasks::THIRTY_BIT;
        // back to handling the junk in this way
        if !self.start {
            return;
        }
        self.events[self.current_event].end_event(event_num);
        self.current_event += 1;
        self.stop = true;
        self.start = false;
    }

    // see if this is real data or dummy events/extended timestamp
    fn check_subheader(&mut self, data_word: u32) -> bool {
        let subheader = data_word >> 28 & bitmasks::THREE_BIT;
        match subheader {
            0 => false, // dummy event
            1 => true,  // actual data
            2 => {
                // extended timestamp
                let ts = data_word & bitmasks::SIXTEEN_BIT;
                self.events[self.current_event].extended_ts(ts);
                false
            }

            _ => panic!("Invalid subheader in bank word!"),
        }
    }

    // handles the 16/32 qdc logic
    fn parse_qdc(&mut self, nchannels: u32, data_word: u32) {
        if self.check_subheader(data_word) {
            // get the channel
            let channel_mask = if nchannels == 32 {
                bitmasks::SEVEN_BIT
            } else {
                bitmasks::FIVE_BIT
            };
            // find out what the physical channel was that fired and what
            // kind of event
            let mut channel = data_word >> 16 & channel_mask;
            let evt_type = channel / nchannels;
            channel -= evt_type * nchannels;
            // now check what kind of event we have
            match evt_type {
                0 => {
                    self.push_long(nchannels, channel, data_word);
                }

                1 => {
                    self.push_tdc(nchannels, channel, data_word);
                }

                2 => self.push_trigger_dt(nchannels, channel, data_word),
                3 => {
                    self.push_short(nchannels, channel, data_word);
                }
                _ => panic!("Unknown event type!!"),
            }
        }
    }

    // handles the 16/32 scp logic
    fn parse_scp(&mut self, nchannels: u32, data_word: u32) {
        if self.check_subheader(data_word) {
            // get the channel
            let channel_mask = if nchannels == 32 {
                bitmasks::SEVEN_BIT
            } else {
                bitmasks::FIVE_BIT
            };
            // find out what the physical channel was that fired and what
            // kind of event
            let mut channel = data_word >> 16 & channel_mask;
            let evt_type = channel / nchannels;
            channel -= evt_type * nchannels;
            // now check what kind of event we have
            match evt_type {
                0 => {
                    self.push_adc(nchannels, channel, data_word);
                }

                1 => {
                    self.push_tdc(nchannels, channel, data_word);
                }

                2 => self.push_trigger_dt(nchannels, channel, data_word),
                _ => panic!("Unknown event type!!"),
            }
        }
    }

    // These update the events
    fn push_adc(&mut self, nchannels: u32, channel: u32, data_word: u32) {
        let adc = data_word & bitmasks::SIXTEEN_BIT;
        let pile_up = match nchannels {
            16 => (data_word >> 23 & bitmasks::ONE_BIT) != 0,
            32 => (data_word >> 24 & bitmasks::ONE_BIT) != 0,
            _ => panic!("Invalid number of channels: {}", nchannels),
        };
        self.events[self.current_event].add_adc(channel, adc, pile_up);
    }

    fn push_tdc(&mut self, _nchannels: u32, channel: u32, data_word: u32) {
        let tdc = data_word & bitmasks::SIXTEEN_BIT;
        self.events[self.current_event].add_tdc(channel, tdc);
    }

    fn push_long(&mut self, _nchannels: u32, channel: u32, data_word: u32) {
        let long_value = data_word & bitmasks::SIXTEEN_BIT;
        self.events[self.current_event].add_long(channel, long_value);
    }

    fn push_short(&mut self, _nchannels: u32, channel: u32, data_word: u32) {
        let short_value = data_word & bitmasks::SIXTEEN_BIT;
        self.events[self.current_event].add_short(channel, short_value);
    }

    fn push_trigger_dt(&mut self, _nchannels: u32, channel: u32, data_word: u32) {
        let trigger_dt = data_word & bitmasks::SIXTEEN_BIT;
        self.events[self.current_event].add_trigger_dt(channel, trigger_dt);
    }

    pub fn clear_data(&mut self) {
        self.current_event = 0;
        self.events.clear();
    }
}
