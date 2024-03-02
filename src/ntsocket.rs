use crate::attr::Nl80211Attr;
use crate::attr::*;
use crate::channels::*;
use crate::cmd::Nl80211Cmd;
use crate::interface::Interface;
use crate::phy::Frequency;
use crate::phy::WirelessPhy;
use crate::util::decode_iftypes;
use crate::{NL_80211_GENL_NAME, NL_80211_GENL_VERSION};
use neli::attr::{AttrHandle, Attribute};
use neli::consts::{nl::NlmF, nl::NlmFFlags, nl::Nlmsg, socket::NlFamily};
use neli::genl::AttrType;
use neli::genl::{Genlmsghdr, Nlattr};
use neli::nl::{NlPayload, Nlmsghdr};
use neli::socket::NlSocketHandle;
use neli::types::{Buffer, GenlBuffer};

use std::collections::HashMap;
use std::fmt;
use std::fs;

/// A generic netlink socket to send commands and receive messages
pub struct NtSocket {
    pub(crate) sock: NlSocketHandle,
    pub(crate) family_id: u16,
}

impl fmt::Debug for NtSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NtSocket")
            .field("sock", &format_args!("<NlSocketHandle>"))
            .field("family_id", &self.family_id)
            .finish()
    }
}

impl NtSocket {
    /// Create a new nl80211 socket with netlink
    pub fn connect() -> Result<Self, String> {
        let mut sock =
            NlSocketHandle::connect(NlFamily::Generic, None, &[]).map_err(|e| e.to_string())?;
        sock.nonblock().map_err(|e| e.to_string())?;
        let family_id = sock
            .resolve_genl_family(NL_80211_GENL_NAME)
            .map_err(|e| e.to_string())?;
        Ok(Self { sock, family_id })
    }

    pub fn cmd_get_interfaces(&mut self) -> Result<HashMap<u32, Interface>, String> {
        let msghdr = Genlmsghdr::<Nl80211Cmd, Nl80211Attr>::new(
            Nl80211Cmd::CmdGetInterface,
            NL_80211_GENL_VERSION,
            GenlBuffer::new(),
        );

        let nlhdr: Nlmsghdr<u16, Genlmsghdr<Nl80211Cmd, Nl80211Attr>> = {
            let len = None;
            let nl_type = self.family_id;
            let flags = NlmFFlags::new(&[NlmF::Request, NlmF::Dump]);
            let seq = None;
            let pid = None;
            let payload = NlPayload::Payload(msghdr);
            Nlmsghdr::new(len, nl_type, flags, seq, pid, payload)
        };

        self.sock
            .send(nlhdr)
            .map_err(|e| format!("CMD_GET_INTERFACES {}", e))?;

        let iter = self
            .sock
            .iter::<Nlmsg, Genlmsghdr<Nl80211Cmd, Nl80211Attr>>(false);

        let mut retval: HashMap<u32, Interface> = HashMap::new();
        for res in iter {
            match res {
                Ok(response) => {
                    match response.nl_type {
                        Nlmsg::Noop => (),
                        Nlmsg::Error => return Err("Error (CMD_GET_INTERFACES)".to_string()),
                        Nlmsg::Done => break,
                        _ => {
                            if let Some(p) = response.nl_payload.get_payload() {
                                if p.cmd != Nl80211Cmd::CmdNewInterface {
                                    continue;
                                }
                                let handle = p.get_attr_handle();
                                let mut freq: Frequency = Frequency::default();
        
                                let wiphy: u32 = handle
                                    .get_attribute(Nl80211Attr::AttrWiphy)
                                    .unwrap()
                                    .get_payload_as()
                                    .unwrap();
        
                                let mut interface = Interface::new(wiphy);
        
                                // Get iftype
                                let iftype_payload: u32 = handle
                                    .get_attribute(Nl80211Attr::AttrIftype)
                                    .unwrap()
                                    .get_payload_as()
                                    .unwrap();
        
                                let lsb: u8 = (iftype_payload & 0xFF) as u8;
        
                                let iftype =
                                    Nl80211Iftype::from_u8(lsb).unwrap_or(Nl80211Iftype::IftypeUnspecified);
                                interface.current_iftype = Some(iftype);
        
                                // Iterate other attributes
                                for attr in handle.iter() {
                                    match attr.nla_type.nla_type {
                                        // IfIndex (eg: wlan0)
                                        Nl80211Attr::AttrIfindex => {
                                            interface.index =
                                                Some(attr.get_payload_as().map_err(|err| err.to_string())?);
                                        }
                                        // IFNAME (eg: wlan0)
                                        Nl80211Attr::AttrIfname => {
                                            interface.name = Some(
                                                attr.get_payload_as_with_len()
                                                    .map_err(|err| err.to_string())?,
                                            );
                                        }
                                        // Mac Address of the interface
                                        Nl80211Attr::AttrMac => {
                                            let mut mac = Vec::new();
                                            let vecmac: Vec<u8> = attr
                                                .get_payload_as_with_len()
                                                .map_err(|err| err.to_string())?;
                                            for byte in vecmac {
                                                mac.push(byte);
                                            }
        
                                            interface.mac = Some(mac);
                                        }
                                        // The SSID the interface is associated with
                                        Nl80211Attr::AttrSsid => {
                                            interface.ssid = Some(
                                                attr.get_payload_as_with_len()
                                                    .map_err(|err| err.to_string())?,
                                            );
                                        }
                                        // The frequency the wireless interface is using
                                        Nl80211Attr::AttrWiphyFreq => {
                                            freq.frequency =
                                                Some(attr.get_payload_as().map_err(|err| err.to_string())?);
                                            freq.channel = Some(
                                                chan_from_frequency(freq.frequency.unwrap()),
                                            );
        
                                        }
                                        // Channel Type (Width)
                                        Nl80211Attr::AttrChannelWidth => {
                                            freq.width =
                                                Some(attr.get_payload_as().map_err(|err| err.to_string())?);
                                        }
                                        // Transmission Power Level
                                        Nl80211Attr::AttrWiphyTxPowerLevel => {
                                            freq.pwr =
                                                Some(attr.get_payload_as().map_err(|err| err.to_string())?);
                                        }
                                        // Wireless Device
                                        Nl80211Attr::AttrWdev => {
                                            interface.device =
                                                Some(attr.get_payload_as().map_err(|err| err.to_string())?)
                                        }
                                        _ => (),
                                    }
                                }
        
                                interface.frequency = Some(freq);
                                retval.insert(interface.phy_name, interface);
                            }
                        }
                    }
                },
                Err(e) => {
                    return Err(format!("Error ({e})").to_string());
                }
            }
            
        }
        Ok(retval)
    }

    pub fn cmd_get_wiphy(&mut self, phy: u32) -> Result<WirelessPhy, String> {
        let msghdr = Genlmsghdr::<Nl80211Cmd, Nl80211Attr>::new(
            Nl80211Cmd::CmdGetWiphy,
            NL_80211_GENL_VERSION,
            {
                let mut attrs = GenlBuffer::new();
                attrs.push(Nlattr::new(false, false, Nl80211Attr::AttrWiphy, phy).unwrap());
                attrs.push(Nlattr {
                    nla_len: 4,
                    nla_type: AttrType {
                        nla_nested: false,
                        nla_network_order: false,
                        nla_type: Nl80211Attr::AttrSplitWiphyDump,
                    },
                    nla_payload: Buffer::new(),
                });
                attrs
            },
        );

        let nlhdr: Nlmsghdr<u16, Genlmsghdr<Nl80211Cmd, Nl80211Attr>> = {
            let len = None;
            let nl_type = self.family_id;
            let flags = NlmFFlags::new(&[NlmF::Request, NlmF::Ack, NlmF::Root, NlmF::Match]);
            let seq = None;
            let pid = Some(42069);
            let payload = NlPayload::Payload(msghdr);
            Nlmsghdr::new(len, nl_type, flags, seq, pid, payload)
        };

        self.sock.send(nlhdr).map_err(|err| err.to_string())?;

        let iter = self
            .sock
            .iter::<Nlmsg, Genlmsghdr<Nl80211Cmd, Nl80211Attr>>(false);

        let mut phy = WirelessPhy::default();

        for response in iter {
            let response = response.unwrap();
            match response.nl_type {
                Nlmsg::Noop => (),
                Nlmsg::Error => panic!("Error with netlink during cmd_get_split_wiphy()"),
                Nlmsg::Done => break,
                _ => {
                    if let Some(p) = response.nl_payload.get_payload() {
                        if p.cmd != Nl80211Cmd::CmdNewWiphy {
                            continue;
                        }
                        let handle = p.get_attr_handle();
                        let wiphy: u32 = handle
                            .get_attribute(Nl80211Attr::AttrWiphy)
                            .unwrap()
                            .get_payload_as()
                            .unwrap();
                        phy.phy = wiphy;

                        let wiphy_name: String = handle
                            .get_attribute(Nl80211Attr::AttrWiphyName)
                            .unwrap()
                            .get_payload_as_with_len()
                            .map_err(|err| err.to_string())?;
                        phy.phy_name = Some(wiphy_name.clone());

                        let driver_path =
                            format!("/sys/class/ieee80211/{}/device/driver", wiphy_name.clone());

                        if let Ok(link_path) = fs::read_link(&driver_path) {
                            if let Some(driver_name) = link_path.file_name() {
                                if let Some(driver_name_str) = driver_name.to_str() {
                                    phy.driver = Some(driver_name_str.to_string());
                                }
                            }
                        }

                        for attr in handle.get_attrs() {
                            match attr.nla_type.nla_type {
                                Nl80211Attr::AttrWiphyFreq => {
                                    phy.frequency.frequency =
                                        Some(attr.get_payload_as().map_err(|err| err.to_string())?);
                                    phy.frequency.channel = Some(
                                        chan_from_frequency(phy.frequency.frequency.unwrap()),
                                    );
                                }
                                // Channel Type (Width)
                                Nl80211Attr::AttrChannelWidth => {
                                    phy.frequency.width =
                                        Some(attr.get_payload_as().map_err(|err| err.to_string())?);

                                }
                                // Transmission Power Level
                                Nl80211Attr::AttrWiphyTxPowerLevel => {
                                    phy.frequency.pwr =
                                        Some(attr.get_payload_as().map_err(|err| err.to_string())?);

                                }
                                Nl80211Attr::AttrSupportedIftypes => {
                                    let payload = attr
                                        .get_payload_as_with_len()
                                        .map_err(|err| err.to_string())?;
                                    phy.iftypes = Some(decode_iftypes(payload));
                                    phy.has_netlink = Some(true);
                                }
                                Nl80211Attr::AttrWiphyBands => {
                                    let handle: AttrHandle<
                                        '_,
                                        GenlBuffer<Nl80211Bandc, Buffer>,
                                        Nlattr<Nl80211Bandc, Buffer>,
                                    > = attr.get_attr_handle().unwrap();
                                    let bands = handle.get_attrs();
                                    let mut supported_bands: Vec<BandList> = Vec::new();

                                    for band in bands {
                                        let mut bandlist = BandList::default();

                                        match band.nla_type.nla_type {
                                            Nl80211Bandc::Band2ghz => {
                                                bandlist.band = WiFiBand::Band2GHz
                                            }
                                            Nl80211Bandc::Band5ghz => {
                                                bandlist.band = WiFiBand::Band5GHz
                                            }
                                            Nl80211Bandc::Band60ghz => {
                                                bandlist.band = WiFiBand::Band60GHz
                                            }
                                            Nl80211Bandc::Band6ghz => {
                                                bandlist.band = WiFiBand::Band6GHz
                                            }
                                            Nl80211Bandc::BandS1ghz => {}
                                            Nl80211Bandc::BandLC => {}
                                            Nl80211Bandc::UnrecognizedConst(_) => {}
                                        }


                                        let bandhandle = band.get_attr_handle().unwrap();

                                        
                                        for bandattr in bandhandle.get_attrs() {
                                            match bandattr.nla_type.nla_type {
                                                Nl80211BandAttr::BandAttrFreqs => {
                                                    let freqhandle: AttrHandle<
                                                        '_,
                                                        GenlBuffer<u16, Buffer>,
                                                        Nlattr<u16, Buffer>,
                                                    > = bandattr.get_attr_handle().unwrap();
                                                    let mut channels: Vec<ChannelData> =
                                                        [].to_vec();
                                                    for freq in freqhandle.get_attrs() {
                                                        let freqdata_handle: AttrHandle<
                                                            '_,
                                                            GenlBuffer<
                                                                Nl80211FrequencyAttr,
                                                                Buffer,
                                                            >,
                                                            Nlattr<Nl80211FrequencyAttr, Buffer>,
                                                        > = freq.get_attr_handle().unwrap();
                                                        let mut channel: ChannelData =
                                                            ChannelData::default();
                                                        for freqattr in freqdata_handle.get_attrs()
                                                        {
                                                            match freqattr.nla_type.nla_type {
                                                                    Nl80211FrequencyAttr::FrequencyAttrFreq => {
                                                                        let frequency: u32 = freqattr.get_payload_as().map_err(|err| err.to_string())?;
                                                                        channel.frequency = frequency;
                                                                        channel.channel = chan_from_frequency(frequency)
                                                                        
                                                                    }
                                                                    Nl80211FrequencyAttr::FrequencyAttrDisabled => {
                                                                        channel.status = FrequencyStatus::Disabled;
                                                                    },
                                                                    Nl80211FrequencyAttr::FrequencyAttrMaxTxPower => {
                                                                        channel.pwr = freqattr.get_payload_as().map_err(|err| err.to_string())?;
                                                                    },
                                                                    _ => {}
                                                                }
                                                        }
                                                        channels.push(channel);
                                                    }
                                                    bandlist.channels = channels;
                                                }
                                                Nl80211BandAttr::BandAttrInvalid => {}
                                                Nl80211BandAttr::BandAttrRates => {}
                                                Nl80211BandAttr::BandAttrHtMcsSet => {}
                                                Nl80211BandAttr::BandAttrHtCapa => {}
                                                Nl80211BandAttr::BandAttrHtAmpduFactor => {}
                                                Nl80211BandAttr::BandAttrHtAmpduDensity => {}
                                                Nl80211BandAttr::BandAttrVhtMcsSet => {}
                                                Nl80211BandAttr::BandAttrVhtCapa => {}
                                                Nl80211BandAttr::UnrecognizedConst(_) => {}
                                            }
                                        }
                                        supported_bands.push(bandlist);
                                    }
                                    if !supported_bands.is_empty() {
                                        let frequency_list = phy.frequency_list.get_or_insert_with(Vec::new);
                                    
                                        for new_band in supported_bands {
                                            if let Some(existing_band) = frequency_list.iter_mut().find(|b| b.band == new_band.band) {
                                                existing_band.channels.extend(new_band.channels);
                                            } else {
                                                frequency_list.push(new_band);
                                            }
                                        }
                                    }
                                }
                                Nl80211Attr::AttrFeatureFlags => {
                                    const NL80211_FEATURE_ACTIVE_MONITOR: u32 = 1 << 17;
                                    let feature_flags: u32 =
                                        attr.get_payload_as().map_err(|err| err.to_string())?;
                                    if feature_flags & NL80211_FEATURE_ACTIVE_MONITOR != 0 {
                                        phy.active_monitor = Some(true);
                                    } else {
                                        phy.active_monitor = Some(false);
                                    }
                                }
                                Nl80211Attr::AttrIftype => {
                                    let payload =
                                        attr.get_payload_as().map_err(|err| err.to_string())?;
                                    //println!("AttrIftype: {:#?}", payload);
                                    phy.current_iftype = Some(payload);
                                }
                                _ => {} // TODO implement other attributes
                            }
                        }
                    }
                }
            }
        }
        Ok(phy)
    }

    /// To protect against too much data (returning empty channels) this will return the phy's and then call each one independently
    pub fn cmd_get_all_wiphy(&mut self) -> Result<HashMap<u32, WirelessPhy>, String> {
        let msghdr = Genlmsghdr::<Nl80211Cmd, Nl80211Attr>::new(
            Nl80211Cmd::CmdGetWiphy,
            NL_80211_GENL_VERSION,
            GenlBuffer::new(),
        );

        let nlhdr: Nlmsghdr<u16, Genlmsghdr<Nl80211Cmd, Nl80211Attr>> = {
            let len = None;
            let nl_type = self.family_id;
            let flags = NlmFFlags::new(&[NlmF::Request, NlmF::Dump]);
            let seq = None;
            let pid = None;
            let payload = NlPayload::Payload(msghdr);
            Nlmsghdr::new(len, nl_type, flags, seq, pid, payload)
        };

        self.sock
            .send(nlhdr)
            .map_err(|err| format!("CMD_GET_WIPHY {}", err))?;

        let iter = self
            .sock
            .iter::<Nlmsg, Genlmsghdr<Nl80211Cmd, Nl80211Attr>>(false);

        let mut phys: HashMap<u32, WirelessPhy> = HashMap::new();
        let mut phys_available: Vec<u32> = Vec::new();

        for response in iter {
            let response = response.unwrap();
            match response.nl_type {
                Nlmsg::Noop => (),
                Nlmsg::Error => panic!("Error with netlink during cmd_get_split_wiphy()"),
                Nlmsg::Done => break,
                _ => {
                    if let Some(p) = response.nl_payload.get_payload() {
                        if p.cmd != Nl80211Cmd::CmdNewWiphy {
                            continue;
                        }
                        let handle = p.get_attr_handle();
                        let wiphy: u32 = handle
                            .get_attribute(Nl80211Attr::AttrWiphy)
                            .unwrap()
                            .get_payload_as()
                            .unwrap();
                        phys_available.push(wiphy);
                    }
                }
            }
        }
        for phy in phys_available {
            if let Ok(phy_data) = self.cmd_get_wiphy(phy) {
                phys.insert(phy, phy_data);
            } else {
                continue;
            }
        }
        Ok(phys)
    }

    pub fn set_type_vec(
        &mut self,
        interface_index: u32,
        iftype: Nl80211Iftype,
        active: bool,
    ) -> Result<(), String> {
        let msghdr = Genlmsghdr::<Nl80211Cmd, Nl80211Attr>::new(
            Nl80211Cmd::CmdSetInterface,
            NL_80211_GENL_VERSION,
            {
                let mut attrs = GenlBuffer::new();
                attrs.push(
                    Nlattr::new(false, false, Nl80211Attr::AttrIfindex, interface_index).unwrap(),
                );
                attrs.push(
                    Nlattr::new(
                        false,
                        false,
                        Nl80211Attr::AttrIftype,
                        u16::from(iftype) as u32,
                    )
                    .unwrap(),
                );
                if iftype == Nl80211Iftype::IftypeMonitor && active {
                    attrs.push(
                        Nlattr::new(
                            false,
                            false,
                            Nl80211Attr::AttrMntrFlags,
                            Nlattr {
                                nla_len: 4,
                                nla_type: AttrType {
                                    nla_nested: false,
                                    nla_network_order: false,
                                    nla_type: Nl80211MntrFlags::MntrFlagActive,
                                },
                                nla_payload: Buffer::new(),
                            },
                        )
                        .unwrap(),
                    );
                }
                attrs
            },
        );

        let nlhdr: Nlmsghdr<u16, Genlmsghdr<Nl80211Cmd, Nl80211Attr>> = {
            let len = None;
            let nl_type = self.family_id;
            let flags = NlmFFlags::new(&[NlmF::Request, NlmF::Ack]);
            let seq = None;
            let pid = None;
            let payload = NlPayload::Payload(msghdr);
            Nlmsghdr::new(len, nl_type, flags, seq, pid, payload)
        };

        // Send the Netlink message
        self.sock.send(nlhdr).map_err(|err| err.to_string())?;

        let iter = self
            .sock
            .iter::<Nlmsg, Genlmsghdr<Nl80211Cmd, Nl80211Attr>>(true);

        for response in iter.flatten() {
            match response.nl_type {
                Nlmsg::Noop => (),
                Nlmsg::Error => match response.nl_payload {
                    NlPayload::Ack(_ack) => continue,
                    NlPayload::Err(err) => {
                        return Err(err.to_string());
                    }
                    NlPayload::Payload(p) => {
                        return Err(format!("{:?}", p));
                    }
                    NlPayload::Empty => {
                        return Err("Payload was empty".to_string());
                    }
                },
                Nlmsg::Done => break,
                _ => (), //println!("Response: {:#?}", response.nl_payload),
            }
        }
        Ok(())
    }

    pub fn set_powersave_off(&mut self, interface_index: u32) -> Result<(), String> {
        let gmsghdr = Genlmsghdr::<Nl80211Cmd, Nl80211Attr>::new(
            Nl80211Cmd::CmdSetWiphy,
            NL_80211_GENL_VERSION,
            {
                let mut attrs = GenlBuffer::new();
                attrs.push(
                    Nlattr::new(false, false, Nl80211Attr::AttrIfindex, interface_index).unwrap(),
                );
                attrs.push(
                    Nlattr::new(
                        false,
                        false,
                        Nl80211Attr::AttrPsState,
                        Nl80211PsState::PsDisabled,
                    )
                    .unwrap(),
                );
                attrs
            },
        );

        let nlhdr: Nlmsghdr<u16, Genlmsghdr<Nl80211Cmd, Nl80211Attr>> = {
            let len = None;
            let nl_type = self.family_id;
            let flags = NlmFFlags::new(&[NlmF::Request, NlmF::Ack]);
            let seq = None;
            let pid = None;
            let payload = NlPayload::Payload(gmsghdr);
            Nlmsghdr::new(len, nl_type, flags, seq, pid, payload)
        };

        // Send the Netlink message
        self.sock.send(nlhdr).map_err(|err| err.to_string())?;

        let iter = self
            .sock
            .iter::<Nlmsg, Genlmsghdr<Nl80211Cmd, Nl80211Attr>>(false);

        for response in iter.flatten() {
            match response.nl_type {
                Nlmsg::Noop => (),
                Nlmsg::Error => match response.nl_payload {
                    NlPayload::Ack(_ack) => continue,
                    NlPayload::Err(err) => {
                        return Err(err.to_string());
                    }
                    NlPayload::Payload(p) => {
                        return Err(format!("{:?}", p));
                    }
                    NlPayload::Empty => {
                        return Err("Payload was empty".to_string());
                    }
                },
                Nlmsg::Done => break,
                _ => (),
            }
        }
        Ok(())
    }

    pub fn set_frequency(
        &mut self,
        interface_index: u32,
        frequency: u32,
        chan_width: Nl80211ChanWidth,
        chan_type: Nl80211ChannelType,
    ) -> Result<(), String> {
        let gmsghdr = Genlmsghdr::<Nl80211Cmd, Nl80211Attr>::new(
            Nl80211Cmd::CmdSetChannel,
            NL_80211_GENL_VERSION,
            {
                let mut attrs = GenlBuffer::new();
                attrs.push(
                    Nlattr::new(false, false, Nl80211Attr::AttrIfindex, interface_index).unwrap(),
                );
                attrs.push(
                    Nlattr::new(false, false, Nl80211Attr::AttrWiphyFreq, frequency).unwrap(),
                );
                attrs.push(
                    Nlattr::new(
                        false,
                        false,
                        Nl80211Attr::AttrChannelWidth,
                        u32::from(u16::from(chan_width)),
                    )
                    .unwrap(),
                );
                attrs.push(
                    Nlattr::new(
                        false,
                        false,
                        Nl80211Attr::AttrWiphyChannelType,
                        u32::from(u16::from(chan_type)),
                    )
                    .unwrap(),
                );
                attrs.push(
                    Nlattr::new(false, false, Nl80211Attr::AttrCenterFreq1, frequency).unwrap(),
                );
                attrs
            },
        );

        let nlhdr: Nlmsghdr<u16, Genlmsghdr<Nl80211Cmd, Nl80211Attr>> = {
            let len = None;
            let nl_type = self.family_id;
            let flags = NlmFFlags::new(&[NlmF::Request, NlmF::Ack]);
            let seq = None;
            let pid = None;
            let payload = NlPayload::Payload(gmsghdr);
            Nlmsghdr::new(len, nl_type, flags, seq, pid, payload)
        };

        // Send the Netlink message

        self.sock
            .send(nlhdr)
            .map_err(|err| format!("Send Err: {}", err))?;

        let iter = self
            .sock
            .iter::<Nlmsg, Genlmsghdr<Nl80211Cmd, Nl80211Attr>>(false);

        for response in iter.flatten() {
            match response.nl_type {
                Nlmsg::Noop => (),
                Nlmsg::Error => match response.nl_payload {
                    NlPayload::Ack(_ack) => continue,
                    NlPayload::Err(err) => {
                        return Err(format!("NlPayload::Err {}", err));
                    }
                    NlPayload::Payload(p) => {
                        return Err(format!("NlPayload::Payload {:?}", p));
                    }
                    NlPayload::Empty => {
                        return Err("Payload was empty".to_string());
                    }
                },
                Nlmsg::Done => break,
                _ => (),
            }
        }
        Ok(())
    }
}

impl From<NtSocket> for NlSocketHandle {
    /// Returns the underlying generic netlink socket
    fn from(sock: NtSocket) -> Self {
        sock.sock
    }
}
