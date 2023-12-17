use std::{
    cmp::Ordering,
    fmt::{self, Write},
};

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum WiFiBand {
    Band2GHz,
    Band5GHz,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ChannelData {
    pub frequency: u32,
    pub channel: WiFiChannel,
    pub pwr: u32,
    pub status: FrequencyStatus,
}

impl Default for ChannelData {
    fn default() -> Self {
        ChannelData {
            frequency: 0,
            channel: WiFiChannel::new(1).unwrap(),
            pwr: 0,
            status: FrequencyStatus::Enabled,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BandList {
    pub band: WiFiBand,
    pub channels: Vec<ChannelData>,
}

impl Default for BandList {
    fn default() -> Self {
        BandList {
            band: WiFiBand::Band2GHz, // Default to 2GHz band
            channels: Vec::new(),
        }
    }
}

pub fn pretty_print_band_lists(band_lists: &[BandList]) -> String {
    let mut output = String::new();
    for band_list in band_lists {
        output += &format!("{:?}:\n", band_list.band);
        let mut line = String::new();
        let mut count = 0;
        for channel in &band_list.channels {
            if channel.status == FrequencyStatus::Enabled {
                let channel_str = match channel.channel {
                    WiFiChannel::Channel2GHz(ch) => format!("{}", ch),
                    WiFiChannel::Channel5GHz(ch) => format!("{}", ch),
                };

                line += &format!("    [{} ({})]", channel.frequency, channel_str);
                count += 1;

                if count % 6 == 0 {
                    line += "\n";
                }
            }
        }
        if !line.ends_with('\n') {
            line += "\n";
        }
        line += "\n";
        output += &line;
    }
    output
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrequencyStatus {
    Disabled,
    Enabled,
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum WiFiChannel {
    Channel2GHz(u8),
    Channel5GHz(u8),
}

#[allow(clippy::incorrect_partial_ord_impl_on_ord_type)]
impl PartialOrd for WiFiChannel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (WiFiChannel::Channel2GHz(freq1), WiFiChannel::Channel2GHz(freq2)) => {
                freq1.partial_cmp(freq2)
            }
            (WiFiChannel::Channel5GHz(freq1), WiFiChannel::Channel5GHz(freq2)) => {
                freq1.partial_cmp(freq2)
            }
            // Define the ordering between different variants here.
            (WiFiChannel::Channel2GHz(_), WiFiChannel::Channel5GHz(_)) => Some(Ordering::Less),
            (WiFiChannel::Channel5GHz(_), WiFiChannel::Channel2GHz(_)) => Some(Ordering::Greater),
        }
    }
}

impl Ord for WiFiChannel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl fmt::Display for WiFiChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WiFiChannel::Channel2GHz(channel) => write!(f, "2.4GHz {}", channel),
            WiFiChannel::Channel5GHz(channel) => write!(f, "5GHz {}", channel),
        }
    }
}

impl WiFiChannel {
    pub fn new(channel: u8) -> Option<WiFiChannel> {
        match channel {
            1..=14 => Some(WiFiChannel::Channel2GHz(channel)),
            36 | 38 | 40 | 42 | 44 | 46 | 48 | 50 | 52 | 54 | 56 | 58 | 60 | 62 | 64 | 100
            | 102 | 104 | 106 | 108 | 110 | 112 | 114 | 116 | 118 | 120 | 122 | 124 | 126 | 128
            | 130 | 132 | 134 | 136 | 138 | 140 | 142 | 144 | 146 | 148 | 149 | 151 | 153 | 155
            | 157 | 159 | 161 | 165 => Some(WiFiChannel::Channel5GHz(channel)),
            _ => None, // Invalid channel
        }
    }

    pub fn get_channel_number(&self) -> u8 {
        match self {
            WiFiChannel::Channel2GHz(channel) => *channel,
            WiFiChannel::Channel5GHz(channel) => *channel,
        }
    }

    pub fn short_string(&self) -> String {
        match self {
            WiFiChannel::Channel2GHz(channel) => format!("{}", channel),
            WiFiChannel::Channel5GHz(channel) => format!("{}", channel),
        }
    }

    pub fn to_frequency(&self) -> Option<u32> {
        match self {
            WiFiChannel::Channel2GHz(channel) => match channel {
                1 => Some(2412),
                2 => Some(2417),
                3 => Some(2422),
                4 => Some(2427),
                5 => Some(2432),
                6 => Some(2437),
                7 => Some(2442),
                8 => Some(2447),
                9 => Some(2452),
                10 => Some(2457),
                11 => Some(2462),
                12 => Some(2467),
                13 => Some(2472),
                14 => Some(2484), // Typically Japan only
                _ => None,        // Invalid channel
            },
            WiFiChannel::Channel5GHz(channel) => {
                // General formula for 5 GHz channels
                Some(5000 + ((*channel as u32) * 5))
            }
        }
    }

    pub fn from_frequency(frequency: u32) -> Option<WiFiChannel> {
        match frequency {
            //2.4 GHz Channels
            2412 => Some(WiFiChannel::Channel2GHz(1)),
            2417 => Some(WiFiChannel::Channel2GHz(2)),
            2422 => Some(WiFiChannel::Channel2GHz(3)),
            2427 => Some(WiFiChannel::Channel2GHz(4)),
            2432 => Some(WiFiChannel::Channel2GHz(5)),
            2437 => Some(WiFiChannel::Channel2GHz(6)),
            2442 => Some(WiFiChannel::Channel2GHz(7)),
            2447 => Some(WiFiChannel::Channel2GHz(8)),
            2452 => Some(WiFiChannel::Channel2GHz(9)),
            2457 => Some(WiFiChannel::Channel2GHz(10)),
            2462 => Some(WiFiChannel::Channel2GHz(11)),
            2467 => Some(WiFiChannel::Channel2GHz(12)),
            2472 => Some(WiFiChannel::Channel2GHz(13)),
            2484 => Some(WiFiChannel::Channel2GHz(14)), // Typically Japan only

            // 4.9GHz
            4920 => Some(WiFiChannel::Channel5GHz(184)),
            4940 => Some(WiFiChannel::Channel5GHz(188)),
            4960 => Some(WiFiChannel::Channel5GHz(192)),
            4980 => Some(WiFiChannel::Channel5GHz(196)),

            // Standard 5 GHz frequencies
            5180 => Some(WiFiChannel::Channel5GHz(36)),
            5190 => Some(WiFiChannel::Channel5GHz(38)),
            5200 => Some(WiFiChannel::Channel5GHz(40)),
            5210 => Some(WiFiChannel::Channel5GHz(42)),
            5220 => Some(WiFiChannel::Channel5GHz(44)),
            5230 => Some(WiFiChannel::Channel5GHz(46)),
            5240 => Some(WiFiChannel::Channel5GHz(48)),
            5250 => Some(WiFiChannel::Channel5GHz(50)),
            5260 => Some(WiFiChannel::Channel5GHz(52)),
            5270 => Some(WiFiChannel::Channel5GHz(54)),
            5280 => Some(WiFiChannel::Channel5GHz(56)),
            5290 => Some(WiFiChannel::Channel5GHz(58)),
            5300 => Some(WiFiChannel::Channel5GHz(60)),
            5310 => Some(WiFiChannel::Channel5GHz(62)),
            5320 => Some(WiFiChannel::Channel5GHz(64)),
            5340 => Some(WiFiChannel::Channel5GHz(68)),
            5360 => Some(WiFiChannel::Channel5GHz(72)),
            5380 => Some(WiFiChannel::Channel5GHz(76)),
            5400 => Some(WiFiChannel::Channel5GHz(80)),
            5420 => Some(WiFiChannel::Channel5GHz(84)),
            5440 => Some(WiFiChannel::Channel5GHz(88)),
            5460 => Some(WiFiChannel::Channel5GHz(92)),
            5480 => Some(WiFiChannel::Channel5GHz(96)),

            // Higher 5 GHz frequencies
            5500 => Some(WiFiChannel::Channel5GHz(100)),
            5510 => Some(WiFiChannel::Channel5GHz(102)),
            5520 => Some(WiFiChannel::Channel5GHz(104)),
            5530 => Some(WiFiChannel::Channel5GHz(106)),
            5540 => Some(WiFiChannel::Channel5GHz(108)),
            5550 => Some(WiFiChannel::Channel5GHz(110)),
            5560 => Some(WiFiChannel::Channel5GHz(112)),
            5570 => Some(WiFiChannel::Channel5GHz(114)),
            5580 => Some(WiFiChannel::Channel5GHz(116)),
            5590 => Some(WiFiChannel::Channel5GHz(118)),
            5600 => Some(WiFiChannel::Channel5GHz(120)),
            5610 => Some(WiFiChannel::Channel5GHz(122)),
            5620 => Some(WiFiChannel::Channel5GHz(124)),
            5630 => Some(WiFiChannel::Channel5GHz(126)),
            5640 => Some(WiFiChannel::Channel5GHz(128)),
            5650 => Some(WiFiChannel::Channel5GHz(130)),
            5660 => Some(WiFiChannel::Channel5GHz(132)),
            5670 => Some(WiFiChannel::Channel5GHz(134)),
            5680 => Some(WiFiChannel::Channel5GHz(136)),
            5690 => Some(WiFiChannel::Channel5GHz(138)),
            5700 => Some(WiFiChannel::Channel5GHz(140)),
            5720 => Some(WiFiChannel::Channel5GHz(142)),
            5745 => Some(WiFiChannel::Channel5GHz(149)),
            5755 => Some(WiFiChannel::Channel5GHz(151)),
            5765 => Some(WiFiChannel::Channel5GHz(153)),
            5775 => Some(WiFiChannel::Channel5GHz(155)),
            5785 => Some(WiFiChannel::Channel5GHz(157)),
            5795 => Some(WiFiChannel::Channel5GHz(159)),
            5805 => Some(WiFiChannel::Channel5GHz(161)),
            5825 => Some(WiFiChannel::Channel5GHz(165)),
            5845 => Some(WiFiChannel::Channel5GHz(169)),
            5865 => Some(WiFiChannel::Channel5GHz(173)),
            5885 => Some(WiFiChannel::Channel5GHz(177)),
            5905 => Some(WiFiChannel::Channel5GHz(181)),

            _ => None, // Invalid or unsupported frequency
        }
    }
}
