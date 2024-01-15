use std::{
    cmp::Ordering,
    fmt::{self, Write},
};

static CHANNELS_2GHZ_STR: [&str; 14] = [
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
];

static CHANNELS_5GHZ_STR: [&str; 55] = [
    "34", "36", "38", "40", "42", "44", "46", "48", "50", "52", "54", "56", "58", "60", "62", "64",
    "100", "102", "104", "106", "108", "110", "112", "114", "116", "118", "120", "122", "124",
    "126", "128", "130", "132", "134", "136", "138", "140", "142", "144", "149", "151", "153",
    "155", "157", "159", "161", "165", "169", "173", "177", "181", "184", "188", "192", "196",
];

static CHANNELS_6GHZ_STR: [&str; 59] = [
    "1.6e", "5.6e", "9.6e", "13.6e", "17.6e", "21.6e", "25.6e", "29.6e", "33.6e", "37.6e", "41.6e",
    "45.6e", "49.6e", "53.6e", "57.6e", "61.6e", "65.6e", "69.6e", "73.6e", "77.6e", "81.6e",
    "85.6e", "89.6e", "93.6e", "97.6e", "101.6e", "105.6e", "109.6e", "113.6e", "117.6e", "121.6e",
    "125.6e", "129.6e", "133.6e", "137.6e", "141.6e", "145.6e", "149.6e", "153.6e", "157.6e",
    "161.6e", "165.6e", "169.6e", "173.6e", "177.6e", "181.6e", "185.6e", "189.6e", "193.6e",
    "197.6e", "201.6e", "205.6e", "209.6e", "213.6e", "217.6e", "221.6e", "225.6e", "229.6e",
    "233.6e",
];

static CHANNELS_60GHZ_STR: [&str; 6] = ["1.ay", "2.ay", "3.ay", "4.ay", "5.ay", "6.ay"];

static CHANNELS_2GHZ_U8: [u8; 14] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];

static CHANNELS_5GHZ_U8: [u8; 55] = [
    34, 36, 38, 40, 42, 44, 46, 48, 50, 52, 54, 56, 58, 60, 62, 64, 100, 102, 104, 106, 108, 110,
    112, 114, 116, 118, 120, 122, 124, 126, 128, 130, 132, 134, 136, 138, 140, 142, 144, 149, 151,
    153, 155, 157, 159, 161, 165, 169, 173, 177, 181, 184, 188, 192, 196,
];

static CHANNELS_6GHZ_U8: [u8; 59] = [
    1, 5, 9, 13, 17, 21, 25, 29, 33, 37, 41, 45, 49, 53, 57, 61, 65, 69, 73, 77, 81, 85, 89, 93,
    97, 101, 105, 109, 113, 117, 121, 125, 129, 133, 137, 141, 145, 149, 153, 157, 161, 165, 169,
    173, 177, 181, 185, 189, 193, 197, 201, 205, 209, 213, 217, 221, 225, 229, 233,
];

static CHANNELS_60GHZ_U8: [u8; 6] = [1, 2, 3, 4, 5, 6];

fn map_str_to_band_and_channel(channel_str: &str) -> Option<(WiFiBand, u8)> {
    if CHANNELS_2GHZ_STR.contains(&channel_str) {
        channel_str.parse().ok().map(|ch| (WiFiBand::Band2GHz, ch))
    } else if CHANNELS_5GHZ_STR.contains(&channel_str) {
        channel_str.parse().ok().map(|ch| (WiFiBand::Band5GHz, ch))
    } else if CHANNELS_6GHZ_STR.contains(&channel_str) {
        channel_str[..channel_str.len() - 3]
            .parse()
            .ok()
            .map(|ch| (WiFiBand::Band6GHz, ch))
    } else if CHANNELS_60GHZ_STR.contains(&channel_str) {
        channel_str[..channel_str.len() - 3]
            .parse()
            .ok()
            .map(|ch| (WiFiBand::Band60GHz, ch))
    } else {
        None
    }
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum WiFiBand {
    Band2GHz,
    Band5GHz,
    Band6GHz,
    Band60GHz,
}

impl WiFiBand {
    pub fn to_u8(&self) -> u8 {
        match self {
            WiFiBand::Band2GHz => 2,
            WiFiBand::Band5GHz => 5,
            WiFiBand::Band6GHz => 6,
            WiFiBand::Band60GHz => 60,
        }
    }

    pub fn from_u8(band: u8) -> Result<WiFiBand, String> {
        match band {
            2 => Ok(WiFiBand::Band2GHz),
            5 => Ok(WiFiBand::Band5GHz),
            6 => Ok(WiFiBand::Band6GHz),
            60 => Ok(WiFiBand::Band60GHz),
            _ => Err(String::from("BAND NOT FOUND")),
        }
    }
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
            channel: WiFiChannel::new(1, WiFiBand::Band2GHz).unwrap(),
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
                    WiFiChannel::Channel6GHz(ch) => todo!(),
                    WiFiChannel::Channel60GHz(ch) => todo!(),
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
    Channel6GHz(u8),
    Channel60GHz(u8),
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
            (WiFiChannel::Channel2GHz(_), WiFiChannel::Channel6GHz(_)) => Some(Ordering::Less),
            (WiFiChannel::Channel2GHz(_), WiFiChannel::Channel60GHz(_)) => Some(Ordering::Less),
            (WiFiChannel::Channel5GHz(_), WiFiChannel::Channel6GHz(_)) => Some(Ordering::Less),
            (WiFiChannel::Channel5GHz(_), WiFiChannel::Channel60GHz(_)) => Some(Ordering::Less),
            (WiFiChannel::Channel6GHz(_), WiFiChannel::Channel2GHz(_)) => Some(Ordering::Greater),
            (WiFiChannel::Channel6GHz(_), WiFiChannel::Channel5GHz(_)) => Some(Ordering::Greater),
            (WiFiChannel::Channel6GHz(freq1), WiFiChannel::Channel6GHz(freq2)) => {
                freq1.partial_cmp(freq2)
            }
            (WiFiChannel::Channel6GHz(_), WiFiChannel::Channel60GHz(_)) => Some(Ordering::Less),
            (WiFiChannel::Channel60GHz(_), WiFiChannel::Channel2GHz(_)) => Some(Ordering::Greater),
            (WiFiChannel::Channel60GHz(_), WiFiChannel::Channel5GHz(_)) => Some(Ordering::Greater),
            (WiFiChannel::Channel60GHz(_), WiFiChannel::Channel6GHz(_)) => Some(Ordering::Greater),
            (WiFiChannel::Channel60GHz(freq1), WiFiChannel::Channel60GHz(freq2)) => {
                freq1.partial_cmp(freq2)
            }
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
            WiFiChannel::Channel6GHz(channel) => write!(f, "6GHz {}", channel),
            WiFiChannel::Channel60GHz(channel) => write!(f, "60GHz {}", channel),
        }
    }
}

impl WiFiChannel {
    pub fn new(channel: u8, band: WiFiBand) -> Option<WiFiChannel> {
        match band {
            WiFiBand::Band2GHz if CHANNELS_2GHZ_U8.contains(&channel) => {
                Some(WiFiChannel::Channel2GHz(channel))
            }
            WiFiBand::Band5GHz if CHANNELS_5GHZ_U8.contains(&channel) => {
                Some(WiFiChannel::Channel5GHz(channel))
            }
            WiFiBand::Band6GHz if CHANNELS_6GHZ_U8.contains(&channel) => {
                Some(WiFiChannel::Channel6GHz(channel))
            }
            WiFiBand::Band60GHz if CHANNELS_60GHZ_U8.contains(&channel) => {
                Some(WiFiChannel::Channel60GHz(channel))
            }
            _ => None, // Invalid channel or band
        }
    }

    pub fn get_channel_number(&self) -> u8 {
        match self {
            WiFiChannel::Channel2GHz(channel) => *channel,
            WiFiChannel::Channel5GHz(channel) => *channel,
            WiFiChannel::Channel6GHz(channel) => *channel,
            WiFiChannel::Channel60GHz(channel) => *channel,
        }
    }

    pub fn get_default_width(&self) -> u8 {
        match self {
            WiFiChannel::Channel2GHz(channel) => *channel,
            WiFiChannel::Channel5GHz(channel) => *channel,
            WiFiChannel::Channel6GHz(channel) => *channel,
            WiFiChannel::Channel60GHz(channel) => *channel,
        }
    }

    pub fn short_string(&self) -> String {
        match self {
            WiFiChannel::Channel2GHz(channel) => format!("{}", channel),
            WiFiChannel::Channel5GHz(channel) => format!("{}", channel),
            WiFiChannel::Channel6GHz(channel) => format!("{}", channel),
            WiFiChannel::Channel60GHz(channel) => format!("{}", channel),
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
                14 => Some(2484),
                _ => None,
            },
            WiFiChannel::Channel5GHz(channel) => {
                // General formula for 5 GHz channels
                Some(5000 + ((*channel as u32) * 5))
            }
            WiFiChannel::Channel6GHz(channel) => match channel {
                1 => Some(5955),
                5 => Some(5975),
                9 => Some(5995),
                13 => Some(6015),
                17 => Some(6035),
                21 => Some(6055),
                25 => Some(6075),
                29 => Some(6095),
                33 => Some(6115),
                37 => Some(6135),
                41 => Some(6155),
                45 => Some(6175),
                49 => Some(6195),
                53 => Some(6215),
                57 => Some(6235),
                61 => Some(6255),
                65 => Some(6275),
                69 => Some(6295),
                73 => Some(6315),
                77 => Some(6335),
                81 => Some(6355),
                85 => Some(6375),
                89 => Some(6395),
                93 => Some(6415),
                97 => Some(6435),
                101 => Some(6455),
                105 => Some(6475),
                109 => Some(6495),
                113 => Some(6515),
                117 => Some(6535),
                121 => Some(6555),
                125 => Some(6575),
                129 => Some(6595),
                133 => Some(6615),
                137 => Some(6635),
                141 => Some(6655),
                145 => Some(6675),
                149 => Some(6695),
                153 => Some(6715),
                157 => Some(6735),
                161 => Some(6755),
                165 => Some(6775),
                169 => Some(6795),
                173 => Some(6815),
                177 => Some(6835),
                181 => Some(6855),
                185 => Some(6875),
                189 => Some(6895),
                193 => Some(6915),
                197 => Some(6935),
                201 => Some(6955),
                205 => Some(6975),
                209 => Some(6995),
                213 => Some(7015),
                217 => Some(7035),
                221 => Some(7055),
                225 => Some(7075),
                229 => Some(7095),
                233 => Some(7115),
                _ => None,
            },
            WiFiChannel::Channel60GHz(channel) => match channel {
                1 => Some(58320),
                2 => Some(60480),
                3 => Some(62640),
                4 => Some(64800),
                5 => Some(66960),
                6 => Some(69120),
                _ => None,
            },
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
            5170 => Some(WiFiChannel::Channel5GHz(34)),
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

            // 6GHZ
            5955 => Some(WiFiChannel::Channel6GHz(1)),
            5975 => Some(WiFiChannel::Channel6GHz(5)),
            5995 => Some(WiFiChannel::Channel6GHz(9)),
            6015 => Some(WiFiChannel::Channel6GHz(13)),
            6035 => Some(WiFiChannel::Channel6GHz(17)),
            6055 => Some(WiFiChannel::Channel6GHz(21)),
            6075 => Some(WiFiChannel::Channel6GHz(25)),
            6095 => Some(WiFiChannel::Channel6GHz(29)),
            6115 => Some(WiFiChannel::Channel6GHz(33)),
            6135 => Some(WiFiChannel::Channel6GHz(37)),
            6155 => Some(WiFiChannel::Channel6GHz(41)),
            6175 => Some(WiFiChannel::Channel6GHz(45)),
            6195 => Some(WiFiChannel::Channel6GHz(49)),
            6215 => Some(WiFiChannel::Channel6GHz(53)),
            6235 => Some(WiFiChannel::Channel6GHz(57)),
            6255 => Some(WiFiChannel::Channel6GHz(61)),
            6275 => Some(WiFiChannel::Channel6GHz(65)),
            6295 => Some(WiFiChannel::Channel6GHz(69)),
            6315 => Some(WiFiChannel::Channel6GHz(73)),
            6335 => Some(WiFiChannel::Channel6GHz(77)),
            6355 => Some(WiFiChannel::Channel6GHz(81)),
            6375 => Some(WiFiChannel::Channel6GHz(85)),
            6395 => Some(WiFiChannel::Channel6GHz(89)),
            6415 => Some(WiFiChannel::Channel6GHz(93)),
            6435 => Some(WiFiChannel::Channel6GHz(97)),
            6455 => Some(WiFiChannel::Channel6GHz(101)),
            6475 => Some(WiFiChannel::Channel6GHz(105)),
            6495 => Some(WiFiChannel::Channel6GHz(109)),
            6515 => Some(WiFiChannel::Channel6GHz(113)),
            6535 => Some(WiFiChannel::Channel6GHz(117)),
            6555 => Some(WiFiChannel::Channel6GHz(121)),
            6575 => Some(WiFiChannel::Channel6GHz(125)),
            6595 => Some(WiFiChannel::Channel6GHz(129)),
            6615 => Some(WiFiChannel::Channel6GHz(133)),
            6635 => Some(WiFiChannel::Channel6GHz(137)),
            6655 => Some(WiFiChannel::Channel6GHz(141)),
            6675 => Some(WiFiChannel::Channel6GHz(145)),
            6695 => Some(WiFiChannel::Channel6GHz(149)),
            6715 => Some(WiFiChannel::Channel6GHz(153)),
            6735 => Some(WiFiChannel::Channel6GHz(157)),
            6755 => Some(WiFiChannel::Channel6GHz(161)),
            6775 => Some(WiFiChannel::Channel6GHz(165)),
            6795 => Some(WiFiChannel::Channel6GHz(169)),
            6815 => Some(WiFiChannel::Channel6GHz(173)),
            6835 => Some(WiFiChannel::Channel6GHz(177)),
            6855 => Some(WiFiChannel::Channel6GHz(181)),
            6875 => Some(WiFiChannel::Channel6GHz(185)),
            6895 => Some(WiFiChannel::Channel6GHz(189)),
            6915 => Some(WiFiChannel::Channel6GHz(193)),
            6935 => Some(WiFiChannel::Channel6GHz(197)),
            6955 => Some(WiFiChannel::Channel6GHz(201)),
            6975 => Some(WiFiChannel::Channel6GHz(205)),
            6995 => Some(WiFiChannel::Channel6GHz(209)),
            7015 => Some(WiFiChannel::Channel6GHz(213)),
            7035 => Some(WiFiChannel::Channel6GHz(217)),
            7055 => Some(WiFiChannel::Channel6GHz(221)),
            7075 => Some(WiFiChannel::Channel6GHz(225)),
            7095 => Some(WiFiChannel::Channel6GHz(229)),
            7115 => Some(WiFiChannel::Channel6GHz(233)),

            // 60GHZ
            58320 => Some(WiFiChannel::Channel60GHz(1)),
            60480 => Some(WiFiChannel::Channel60GHz(2)),
            62640 => Some(WiFiChannel::Channel60GHz(3)),
            64800 => Some(WiFiChannel::Channel60GHz(4)),
            66960 => Some(WiFiChannel::Channel60GHz(5)),
            69120 => Some(WiFiChannel::Channel60GHz(6)),

            _ => None, // Invalid or unsupported frequency
        }
    }
}
