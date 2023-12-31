use super::attr::{Nl80211ChanWidth, Nl80211Iftype};
use super::channels::BandList;
use super::WiFiChannel;

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
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Frequency {
    pub frequency: Option<u32>,
    pub width: Option<Nl80211ChanWidth>,
    pub channel: Option<WiFiChannel>,
    pub pwr: Option<u32>,
}

impl Default for Frequency {
    fn default() -> Self {
        Frequency {
            frequency: Some(2412),
            width: Some(Nl80211ChanWidth::ChanWidth20Noht),
            channel: Some(WiFiChannel::Channel2GHz(1)),
            pwr: Some(0),
        }
    }
}

impl Frequency {
    pub fn print(&self) -> String {
        if let Some(freq) = self.frequency {
            format!("{} ({})", freq, self.channel.clone().unwrap())
        } else {
            "None".to_string()
        }
    }
}

fn decode_iftypes(bytes: Vec<u8>) -> Vec<Nl80211Iftype> {
    bytes
        .chunks(4)
        .filter_map(|chunk| {
            if chunk.len() == 4 {
                match chunk[2] {
                    0 => Some(Nl80211Iftype::IftypeUnspecified),
                    1 => Some(Nl80211Iftype::IftypeAdhoc),
                    2 => Some(Nl80211Iftype::IftypeStation),
                    3 => Some(Nl80211Iftype::IftypeAp),
                    4 => Some(Nl80211Iftype::IftypeApVlan),
                    6 => Some(Nl80211Iftype::IftypeMonitor),
                    7 => Some(Nl80211Iftype::IftypeMeshPoint),
                    // Add other cases as needed
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect()
}

pub fn iftypes_to_string_list(iftypes: &Vec<Nl80211Iftype>) -> String {
    iftypes
        .iter()
        .map(|iftype| iftype.string())
        .collect::<Vec<&str>>()
        .join(", ")
}
