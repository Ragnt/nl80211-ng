use super::attr::{Nl80211ChanWidth, Nl80211Iftype};
use super::channels::BandList;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WirelessPhy {
    pub phy: u32,
    pub phy_name: Option<String>,
    pub frequency_list: Option<Vec<BandList>>, // Supported frequencies
    pub iftypes: Option<Vec<Nl80211Iftype>>,   // Supported interface types
    pub current_iftype: Option<Nl80211Iftype>, // Current interface type
    pub powerstate: Option<u32>,               // Power state
    pub driver: Option<String>,                // Driver information
    pub has_netlink: Option<bool>,             // If Interface has netlink
    pub active_monitor: Option<bool>,          // If Interface has netlink
    pub frequency: Option<Frequency>,          // If Interface has netlink
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Frequency {
    pub frequency: Option<u32>,
    pub width: Option<Nl80211ChanWidth>,
    pub channel: Option<u32>,
    pub pwr: Option<u32>,
}

impl Default for Frequency {
    fn default() -> Self {
        Frequency {
            frequency: None,
            width: Some(Nl80211ChanWidth::ChanWidth20Noht),
            channel: None,
            pwr: Some(0),
        }
    }
}

impl Frequency {
    pub fn print(&self) -> String {
        if let Some(freq) = self.frequency {
            format!("{} ({})", freq, if let Some(chan) = self.channel {
                chan.to_string()
            } else {
                "Unknown".to_string()
            })
        } else {
            "None".to_string()
        }
    }
}

pub fn iftypes_to_string_list(iftypes: &Vec<Nl80211Iftype>) -> String {
    iftypes
        .iter()
        .map(|iftype| iftype.string())
        .collect::<Vec<&str>>()
        .join(", ")
}
