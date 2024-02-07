use crate::attr::Nl80211Iftype;
use crate::attr::Operstate;
use crate::channels::pretty_print_band_lists;
use crate::channels::BandList;
use crate::channels::FrequencyStatus;
use crate::phy::iftypes_to_string_list;
use crate::phy::Frequency;
use crate::phy::WirelessPhy;
use crate::util::wrap_in_box;
use std::collections::HashMap;

/// A struct representing a wifi interface
#[non_exhaustive]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Interface {
    pub index: Option<u32>,                    // AttrIfindex
    pub ssid: Option<Vec<u8>>,                 // AttrSsid
    pub mac: Option<Vec<u8>>,                  // AttrMac
    pub name: Option<Vec<u8>>,                 // AttrIfname
    pub state: Option<Operstate>,              // Operstate
    pub phy: Option<WirelessPhy>,              // AttrWiphy
    pub phy_name: u32,                         // AttrWiphy
    pub device: Option<u64>,                   // Attr
    pub current_iftype: Option<Nl80211Iftype>, // AttrIftype
    pub frequency: Option<Frequency>,          // PHY's operating frequency
}

impl Interface {
    pub fn new(wiphy: u32) -> Interface {
        Interface {
            index: None,
            ssid: None,
            mac: None,
            name: None,
            state: None,
            phy: None,
            phy_name: wiphy,
            device: None,
            current_iftype: None,
            frequency: None,
        }
    }

    pub fn get_frequency_list_simple(&self) -> Option<HashMap<u8, Vec<u8>>> {
        match &self.phy {
            Some(wirelessphy) => {
                let phy = wirelessphy.clone();
                let mut map: HashMap<u8, Vec<u8>> = HashMap::new();
                let freq_list = phy.frequency_list.unwrap();
                for band in freq_list {
                    let bandu8 = band.band.to_u8();
                    let mut channels: Vec<u8> = Vec::new();
                    for channel in band.channels {
                        if channel.status == FrequencyStatus::Enabled {
                            channels.push(channel.channel.get_channel_number())
                        }
                    }
                    map.insert(bandu8, channels);
                }
                Some(map)
            }
            None => None,
        }
    }

    pub fn name_as_string(&self) -> String {
        let name = self
            .name
            .as_ref()
            .map(|n| String::from_utf8(n.clone()).unwrap_or_else(|_| "Invalid UTF-8".to_string()))
            .unwrap_or("Unknown".to_string());
        let stripped_name = name.strip_suffix('\0');
        stripped_name.unwrap().to_string()
    }

    pub fn index_as_string(&self) -> String {
        if let Some(index) = self.index {
            index.to_string()
        } else {
            "".to_string()
        }
    }

    pub fn driver_as_string(&self) -> String {
        self.phy
            .clone()
            .unwrap()
            .driver
            .as_ref()
            .unwrap_or(&"Unknown".to_string())
            .clone()
    }

    pub fn pretty_print(&self) -> String {
        let mut output = "".to_string();
        let interface_line = format!("Interface: {}", &self.name_as_string());
        let index_driver_line = format!(
            "Index: {} | Driver: {}",
            self.index_as_string(),
            self.driver_as_string()
        );
        let mode_monitor_line = format!(
            "Mode: {:?} | Active Monitor: {:?}",
            self.current_iftype
                .unwrap_or(Nl80211Iftype::IftypeUnspecified),
            self.phy.clone().unwrap().active_monitor.unwrap()
        );
        let modes_line = format!(
            "Modes: {}",
            iftypes_to_string_list(&self.phy.clone().unwrap().iftypes.clone().unwrap())
        );

        let state_line = format!(
            "State: {:?}",
            self.state.clone().unwrap_or(Operstate::Unknown)
        );
        let freq = if let Some(freq) = &self.frequency {
            freq.print()
        } else {
            "None".to_string()
        };
        let frequency_line = format!("Current Frequency: {}", freq);
        let lines = [
            interface_line,
            index_driver_line,
            mode_monitor_line,
            modes_line,
            state_line,
            frequency_line,
        ];
        for line in &lines {
            output.push_str(line);
            output.push('\n');
        }
        output.push_str("Enabled Bands/Channels:\n\n");
        output.push_str(&pretty_print_band_lists(
            &self
                .phy
                .clone()
                .unwrap()
                .frequency_list
                .clone()
                .unwrap_or(vec![BandList::default()]),
        ));

        wrap_in_box(&output)
    }

    pub fn merge_with(&mut self, other: Interface) {
        if self.ssid.is_none() {
            self.ssid = other.ssid;
        }
        if self.mac.is_none() {
            self.mac = other.mac;
        }
        if self.name.is_none() {
            self.name = other.name;
        }
        if self.state.is_none() {
            self.state = other.state;
        }
        if self.device.is_none() {
            self.device = other.device;
        }
        if self.current_iftype.is_none() {
            self.current_iftype = other.current_iftype;
        }
    }
}
