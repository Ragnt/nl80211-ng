pub fn map_str_to_band_and_channel(channel_str: &str) -> Option<(WiFiBand, u32)> {
    if channel_str.ends_with(".6e") {
        channel_str[..channel_str.len() - 3]
            .parse::<u32>()
            .ok()
            .map(|ch| (WiFiBand::Band6GHz, ch))
    } else if channel_str.ends_with(".ay") {
        channel_str[..channel_str.len() - 3]
            .parse::<u32>()
            .ok()
            .map(|ch| (WiFiBand::Band60GHz, ch))
    } else {
        match channel_str.parse::<u32>() {
            Ok(ch) => {
                if ch >= 1 && ch <= 14 {
                    Some((WiFiBand::Band2GHz, ch))
                } else if ch >= 14 {
                    Some((WiFiBand::Band5GHz, ch))
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum WiFiBand {
    Band2GHz,
    Band5GHz,
    Band60GHz,
    Band6GHz,
    Unknown,
}

impl WiFiBand {
    pub fn to_u8(&self) -> u8 {
        match self {
            WiFiBand::Band2GHz => 2,
            WiFiBand::Band5GHz => 5,
            WiFiBand::Band6GHz => 6,
            WiFiBand::Band60GHz => 60,
            WiFiBand::Unknown => 0,
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
    pub channel: u32,
    pub pwr: u32,
    pub status: FrequencyStatus,
}

impl Default for ChannelData {
    fn default() -> Self {
        ChannelData {
            frequency: 0,
            channel: 0,
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

pub fn pretty_print_band_lists(band_lists: &[BandList], width: usize) -> String {
    let mut output = String::new();
    for band_list in band_lists {
        if band_list.channels.iter().any(|channel| channel.status == FrequencyStatus::Enabled)
        {
            output += &format!("{:?}:\n  ", band_list.band);
            let mut line = String::new();
            let mut count = 0;
            for channel in &band_list.channels {
                if channel.status == FrequencyStatus::Enabled {
                    let channel_str = match freq_to_band(channel.frequency) {
                        WiFiBand::Band2GHz => format!("{}", channel.channel),
                        WiFiBand::Band5GHz => format!("{}", channel.channel),
                        WiFiBand::Band60GHz => format!("{}.ay", channel.channel),
                        WiFiBand::Band6GHz => format!("{}.6e", channel.channel),
                        WiFiBand::Unknown => format!("Unknown"),
                    };
                    let chanline = format!("[{} ({})]", channel.frequency, channel_str);
                    line += &format!("{:<17}", chanline);
                    count += 1;

                    if count % width == 0 {
                        line += "\n  ";
                    }
                }
            }
            if !line.ends_with('\n') {
                line += "\n";
            }
            line += "\n";
            output += &line;
        }
    }
    output
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrequencyStatus {
    Disabled,
    Enabled,
}

pub fn chan_to_frequency(chan: u32, band: WiFiBand) -> u32 {
    if chan <= 0 {
        return 0; // not supported
    }

    match band {
        WiFiBand::Band2GHz => {
            if chan == 14 {
                2484
            } else if chan < 14 {
                2407 + chan * 5
            } else {
                0 // not supported
            }
        },
        WiFiBand::Band5GHz => {
            if (182..=196).contains(&chan) {
                4000 + chan * 5
            } else {
                5000 + chan * 5
            }
        },
        WiFiBand::Band6GHz => {
            if chan == 2 {
                5935
            } else if chan <= 253 {
                5950 + chan * 5
            } else {
                0 // not supported
            }
        },
        WiFiBand::Band60GHz => {
            if chan < 7 {
                56160 + chan * 2160
            } else {
                0 // not supported
            }
        },
        WiFiBand::Unknown => 0,
    }
}

pub fn chan_from_frequency(freq: u32) -> u32 {
    if freq < 1000 {
        return 0; // Not supported
    }

    if freq == 2484 { // Band2Ghz
        14
    } else if freq == 5935 { // Band6Ghz
        2
    } else if freq < 2484 { // Band2Ghz
       (freq - 2407) / 5
    } else if (4910..=4980).contains(&freq) { //Band5Ghz
        (freq - 4000) / 5
    } else if freq < 5950 { //Band5Ghz or Band6Ghz !!! SEPERATE THESE
        (freq - 5000) / 5
    } else if freq <= 45000 { //Band60Ghz
        (freq - 5950) / 5
    } else if (58320..=70200).contains(&freq) { //Band60Ghz
        (freq - 56160) / 2160
    } else {
        0 // Not supported
    }
}

pub fn freq_to_band(freq: u32) -> WiFiBand {
    if freq < 1000 {
        WiFiBand::Unknown
    } else if freq == 2484 {
        WiFiBand::Band2GHz
    } else if freq == 5935 {
        WiFiBand::Band6GHz
    } else if freq < 2484 {
        WiFiBand::Band2GHz
    } else if (4910..=4980).contains(&freq) {
        WiFiBand::Band5GHz
    } else if (5150..=5925).contains(&freq) {
        WiFiBand::Band5GHz
    } else if (5925..=7125).contains(&freq) {
        WiFiBand::Band6GHz
    } else if freq <= 45000 {
        WiFiBand::Band60GHz
    } else if (58320..=70200).contains(&freq) {
        WiFiBand::Band60GHz
    } else {
        WiFiBand::Unknown
    }
}

