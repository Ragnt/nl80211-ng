pub mod attr;
pub mod channels;
pub mod cmd;
pub mod interface;
pub mod ntsocket;
pub mod phy;
pub mod rtsocket;
pub mod util;

use attr::{Nl80211ChanWidth, Nl80211ChannelType, Nl80211Iftype, Operstate};
use channels::WiFiChannel;
pub use interface::Interface;
use ntsocket::NtSocket;
use phy::WirelessPhy;
use rtsocket::RtSocket;

use std::collections::HashMap;

pub const NL_80211_GENL_NAME: &str = "nl80211";
pub const NL_80211_GENL_VERSION: u8 = 1;

pub struct Nl80211 {
    pub nt_socket: NtSocket,
    pub rt_socket: RtSocket,
    wirelessphys: HashMap<u32, WirelessPhy>,
    interfaces: HashMap<u32, Interface>,
}

impl Nl80211 {
    /// Creates a new instance of the Nl80211 Struct.
    pub fn new() -> Result<Nl80211, String> {
        let mut nt_socket: NtSocket = NtSocket::connect()?;
        let mut rt_socket: RtSocket = RtSocket::connect()?;

        let wirelessphys: HashMap<u32, phy::WirelessPhy> = nt_socket.cmd_get_all_wiphy()?;
        let mut interfaces: HashMap<u32, Interface> = nt_socket.cmd_get_interfaces()?;

        for (phy, interface) in &mut interfaces {
            if wirelessphys.contains_key(&phy) {
                if let Some(index) = interface.index {
                    interface.phy = wirelessphys.get(&phy).cloned();
                    interface.state = Some(rt_socket.get_interface_status(index)?);
                }
            }
        }

        Ok(Nl80211 {
            nt_socket,
            rt_socket,
            wirelessphys,
            interfaces,
        })
    }

    /// Updates the interfaces and Wiphy lists of the struct.
    fn update_interfaces(&mut self) -> Result<(), String> {
        let wirelessphys: HashMap<u32, phy::WirelessPhy> = self.nt_socket.cmd_get_all_wiphy()?;
        let mut interfaces: HashMap<u32, Interface> = self.nt_socket.cmd_get_interfaces()?;

        for (phy, interface) in &mut interfaces {
            if wirelessphys.contains_key(&phy) {
                if let Some(index) = interface.index {
                    interface.phy = wirelessphys.get(&phy).cloned();
                    interface.state = Some(self.rt_socket.get_interface_status(index)?);
                }
            }
        }
        self.interfaces = interfaces;
        self.wirelessphys = wirelessphys;
        Ok(())
    }

    pub fn pretty_print_interfaces(&mut self) {
        for interface in self.interfaces.values() {
            let string = interface.pretty_print();
            println!("{}", string);
        }
    }

    pub fn get_interfaces(&self) -> &HashMap<u32, Interface> {
        &self.interfaces
    }

    pub fn get_wiphys(&self) -> &HashMap<u32, WirelessPhy> {
        &self.wirelessphys
    }

    pub fn get_mut_interfaces(&mut self) -> &mut HashMap<u32, Interface> {
        &mut self.interfaces
    }

    pub fn get_mut_wiphys(&mut self) -> &mut HashMap<u32, WirelessPhy> {
        &mut self.wirelessphys
    }

    pub fn set_interface_monitor(&mut self, active: bool, index: i32) -> Result<(), String> {
        self.nt_socket
            .set_type_vec(index, Nl80211Iftype::IftypeMonitor, active)?;
        //self.update_interfaces()?;
        Ok(())
    }

    pub fn set_interface_station(&mut self, index: i32) -> Result<(), String> {
        self.nt_socket
            .set_type_vec(index, Nl80211Iftype::IftypeStation, false)?;
        self.update_interfaces()?;
        Ok(())
    }

    pub fn set_interface_chan(&mut self, index: i32, channel: u8) -> Result<(), String> {
        self.nt_socket.set_frequency(
            index,
            WiFiChannel::new(channel).unwrap().to_frequency().unwrap(),
            Nl80211ChanWidth::ChanWidth20Noht,
            Nl80211ChannelType::ChanNoHt,
        )?;
        self.update_interfaces()?;
        Ok(())
    }

    // rtnetlink commands- all use interface index.

    pub fn set_interface_up(&mut self, index: i32) -> Result<(), String> {
        self.rt_socket.set_interface_up(index)?;
        self.update_interfaces()?;
        Ok(())
    }

    pub fn set_interface_down(&mut self, index: i32) -> Result<(), String> {
        self.rt_socket.set_interface_down(index)?;
        self.update_interfaces()?;
        Ok(())
    }

    pub fn set_interface_mac(&mut self, index: i32, mac: &[u8; 6]) -> Result<(), String> {
        self.rt_socket.set_interface_mac(index, mac)?;
        self.update_interfaces()?;
        Ok(())
    }

    pub fn set_interface_mac_random(&mut self, index: i32) -> Result<(), String> {
        self.rt_socket.set_interface_mac_random(index)?;
        self.update_interfaces()?;
        Ok(())
    }

    fn get_interface_state(&mut self, index: i32) -> Result<Operstate, String> {
        self.rt_socket.get_interface_status(index)
    }
}

///
///
/// The following functions are designed to be used more independently than the Nl80211 struct, making them easier to use in multi-threaded code.
/// They can be fired off in a "one-shot" style.
fn get_interfaces_info() -> Result<HashMap<u32, Interface>, String> {
    let mut nt_socket: NtSocket = NtSocket::connect()?;
    let mut rt_socket: RtSocket = RtSocket::connect()?;

    let wiphys: HashMap<u32, phy::WirelessPhy> = nt_socket.cmd_get_all_wiphy()?;
    let mut interfaces: HashMap<u32, Interface> = nt_socket.cmd_get_interfaces()?;

    for (phy, interface) in &mut interfaces {
        if wiphys.contains_key(phy) {
            if let Some(index) = interface.index {
                interface.phy = wiphys.get(&phy).cloned();
                interface.state = Some(rt_socket.get_interface_status(index)?);
            }
        }
    }
    Ok(interfaces)
}

pub fn get_interface_info_idx(interface_index: i32) -> Result<Interface, String> {
    let mut nt_socket: NtSocket = NtSocket::connect()?;
    let mut rt_socket: RtSocket = RtSocket::connect()?;

    let wiphys: HashMap<u32, phy::WirelessPhy> = nt_socket.cmd_get_all_wiphy()?;
    let mut interfaces: HashMap<u32, Interface> = nt_socket.cmd_get_interfaces()?;

    for (phy, interface) in &mut interfaces {
        if let Some(index) = interface.index {
            if wiphys.contains_key(phy) {
                interface.phy = wiphys.get(phy).cloned();
                interface.state = Some(rt_socket.get_interface_status(index)?);
            }
            if index == interface_index {
                return Ok(interface.clone());
            }
        }
    }
    Err("Interface Not Found".to_string())
}

pub fn get_interface_info_name(interface_name: &String) -> Result<Interface, String> {
    let mut nt_socket: NtSocket = NtSocket::connect()?;
    let mut rt_socket: RtSocket = RtSocket::connect()?;

    let wiphys: HashMap<u32, phy::WirelessPhy> = nt_socket.cmd_get_all_wiphy()?;
    let mut interfaces: HashMap<u32, Interface> = nt_socket.cmd_get_interfaces()?;

    for (phy, interface) in &mut interfaces {
        if let Some(index) = interface.index {
            if wiphys.contains_key(phy) {
                interface.phy = wiphys.get(phy).cloned();
                interface.state = Some(rt_socket.get_interface_status(index)?);
            } else {
                return Err("Phy does not exist...".to_string());
            }
            if &interface.name_as_string() == interface_name {
                return Ok(interface.clone());
            }
        }
    }
    Err("Interface Not Found".to_string())
}

pub fn set_interface_monitor(interface_index: i32, active: bool) -> Result<(), String> {
    let mut nt_socket = NtSocket::connect()?;
    nt_socket.set_type_vec(interface_index, Nl80211Iftype::IftypeMonitor, active)?;
    Ok(())
}

pub fn set_interface_station(interface_index: i32) -> Result<(), String> {
    let mut nt_socket = NtSocket::connect()?;
    nt_socket.set_type_vec(interface_index, Nl80211Iftype::IftypeStation, false)?;
    Ok(())
}

pub fn set_interface_chan(interface_index: i32, channel: u8) -> Result<(), String> {
    let mut nt_socket = NtSocket::connect()?;
    nt_socket.set_frequency(
        interface_index,
        WiFiChannel::new(channel).unwrap().to_frequency().unwrap(),
        Nl80211ChanWidth::ChanWidth20Noht,
        Nl80211ChannelType::ChanNoHt,
    )?;
    Ok(())
}

// rtnetlink commands- all use interface index.

pub fn set_interface_up(interface_index: i32) -> Result<(), String> {
    let mut rt_socket = RtSocket::connect()?;
    rt_socket.set_interface_up(interface_index)?;
    Ok(())
}

pub fn set_interface_down(interface_index: i32) -> Result<(), String> {
    let mut rt_socket = RtSocket::connect()?;
    rt_socket.set_interface_down(interface_index)?;
    Ok(())
}

pub fn set_interface_mac(interface_index: i32, mac: &[u8; 6]) -> Result<(), String> {
    let mut rt_socket = RtSocket::connect()?;
    rt_socket.set_interface_mac(interface_index, mac)?;
    Ok(())
}

pub fn set_interface_mac_random(interface_index: i32) -> Result<(), String> {
    let mut rt_socket = RtSocket::connect()?;
    rt_socket.set_interface_mac_random(interface_index)?;
    Ok(())
}
// This should only be called when "updating" an interface, so we won't update it after doing this.
fn get_interface_state(interface_index: i32) -> Result<Operstate, String> {
    let mut rt_socket = RtSocket::connect().map_err(|e| e.to_string())?;
    rt_socket.get_interface_status(interface_index)
}
