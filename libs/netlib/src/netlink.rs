// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Netlink socket implementation

use anyhow::{anyhow, Context, Result};
use std::mem::{size_of, size_of_val};
use std::os::fd::AsRawFd;

// Netlink constants
const NETLINK_ROUTE: u8 = 0;
const RTM_GETLINK: u16 = 18;
const RTM_GETADDR: u16 = 20;
const RTM_GETROUTE: u16 = 26;

const RTMGRP_LINK: u32 = 1;
const RTMGRP_IPV4_ROUTE: u32 = 0x40;

const IFLA_IFNAME: u16 = 3;
const IFLA_ADDRESS: u16 = 6;
const IFLA_MTU: u16 = 4;

const IFA_ADDRESS: u16 = 1;
const IFA_LOCAL: u16 = 2;
const IFA_LABEL: u16 = 3;

const RTA_DST: u16 = 0;
const RTA_GATEWAY: u16 = 1;
const RTA_OIF: u16 = 4;

#[repr(C)]
struct nlmsghdr {
    nlmsg_len: u32,
    nlmsg_type: u16,
    nlmsg_flags: u16,
    nlmsg_seq: u32,
    nlmsg_pid: u32,
}

#[repr(C)]
struct ifinfomsg {
    ifi_family: u8,
    __ifi_pad: u8,
    ifi_type: u16,
    ifi_index: i32,
    ifi_flags: u32,
    ifi_change: u32,
}

#[repr(C)]
struct ifaddrmsg {
    ifa_family: u8,
    ifa_prefixlen: u8,
    ifa_flags: u8,
    ifa_scope: u8,
    ifa_index: u32,
}

#[repr(C)]
struct rtmsg {
    rtm_family: u8,
    rtm_dst_len: u8,
    rtm_src_len: u8,
    rtm_tos: u8,
    rtm_table: u8,
    rtm_protocol: u8,
    rtm_scope: u8,
    rtm_type: u8,
    rtm_flags: u32,
}

#[derive(Debug, Clone)]
pub struct InterfaceInfo {
    pub index: u32,
    pub name: String,
    pub mac_address: Vec<u8>,
    pub mtu: u32,
    pub is_up: bool,
}

#[derive(Debug, Clone)]
pub struct AddressInfo {
    pub interface: String,
    pub address: String,
    pub prefix_len: u8,
    pub scope: u8,
}

#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub destination: String,
    pub gateway: Option<String>,
    pub interface: Option<String>,
    pub metric: u32,
}

pub struct NetlinkSocket {
    fd: libc::c_int,
    seq: u32,
}

impl NetlinkSocket {
    pub fn new() -> Result<Self> {
        let fd = unsafe {
            libc::socket(
                libc::AF_NETLINK,
                libc::SOCK_RAW | libc::SOCK_CLOEXEC,
                NETLINK_ROUTE as i32,
            )
        };

        if fd < 0 {
            return Err(anyhow!("failed to create netlink socket"));
        }

        let mut addr = unsafe { std::mem::zeroed::<libc::sockaddr_nl>() };
        addr.nl_family = libc::AF_NETLINK as u16;

        unsafe {
            if libc::bind(fd, &addr as *const _ as *const libc::sockaddr, size_of::<libc::sockaddr_nl>() as u32) < 0 {
                libc::close(fd);
                return Err(anyhow!("failed to bind netlink socket"));
            }
        }

        Ok(Self { fd, seq: 0 })
    }

    fn send(&mut self, mut hdr: nlmsghdr, payload: &[u8]) -> Result<()> {
        self.seq += 1;
        hdr.nlmsg_seq = self.seq;
        hdr.nlmsg_len = (size_of::<nlmsghdr>() + payload.len()) as u32;

        let mut buf = vec![0u8; hdr.nlmsg_len as usize];
        unsafe {
            std::ptr::copy_nonoverlapping(
                &hdr as *const nlmsghdr as *const u8,
                buf.as_mut_ptr(),
                size_of::<nlmsghdr>(),
            );
            std::ptr::copy_nonoverlapping(
                payload.as_ptr(),
                buf.as_mut_ptr().add(size_of::<nlmsghdr>()),
                payload.len(),
            );
        }

        let addr = unsafe {
            let mut a: libc::sockaddr_nl = std::mem::zeroed();
            a.nl_family = libc::AF_NETLINK as u16;
            a
        };

        unsafe {
            if libc::sendto(
                self.fd,
                buf.as_ptr() as *const libc::c_void,
                buf.len() as usize,
                0,
                &addr as *const _ as *const libc::sockaddr,
                size_of::<libc::sockaddr_nl>() as u32,
            ) < 0
            {
                return Err(anyhow::anyhow!("failed to send netlink message"));
            }
        }

        Ok(())
    }

    fn recv(&self) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; 4096];
        let len = unsafe {
            libc::recv(
                self.fd,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
                0,
            )
        };

        if len < 0 {
            return Err(anyhow::anyhow!("failed to receive netlink message"));
        }

        buf.truncate(len as usize);
        Ok(buf)
    }

    pub fn get_interfaces(&mut self) -> Result<Vec<InterfaceInfo>> {
        let mut interfaces = Vec::new();

        let hdr = nlmsghdr {
            nlmsg_len: 0,
            nlmsg_type: RTM_GETLINK,
            nlmsg_flags: (libc::NLM_F_REQUEST | libc::NLM_F_DUMP) as u16,
            nlmsg_seq: 0,
            nlmsg_pid: 0,
        };

        let ifm = ifinfomsg {
            ifi_family: libc::AF_UNSPEC as u8,
            __ifi_pad: 0,
            ifi_type: 0,
            ifi_index: 0,
            ifi_flags: 0,
            ifi_change: 0,
        };

        let payload = unsafe {
            std::slice::from_raw_parts(&ifm as *const ifinfomsg as *const u8, size_of::<ifinfomsg>())
        };

        self.send(hdr, payload)?;

        loop {
            let buf = self.recv()?;

            let mut offset = 0;
            while offset < buf.len() {
                let nlh = unsafe {
                    &*(buf.as_ptr().add(offset) as *const nlmsghdr)
                };

                if nlh.nlmsg_type == libc::NLMSG_DONE as u16 {
                    return Ok(interfaces);
                }

                if nlh.nlmsg_type == (libc::NLMSG_ERROR as u16) {
                    return Err(anyhow::anyhow!("netlink error"));
                }

                if nlh.nlmsg_type == RTM_GETLINK {
                    if let Ok(info) = Self::parse_interface(&buf[offset..]) {
                        interfaces.push(info);
                    }
                }

                if nlh.nlmsg_flags & libc::NLM_F_MULTI as u16 == 0 {
                    return Ok(interfaces);
                }

                offset += nlh.nlmsg_len as usize;
            }
        }
    }

    fn parse_interface(data: &[u8]) -> Result<InterfaceInfo> {
        let ifm = unsafe {
            &*(data.as_ptr().add(size_of::<nlmsghdr>()) as *const ifinfomsg)
        };

        let mut info = InterfaceInfo {
            index: ifm.ifi_index as u32,
            name: String::new(),
            mac_address: Vec::new(),
            mtu: 0,
            is_up: (ifm.ifi_flags & 0x1) != 0, // IFF_UP
        };

        let mut offset = size_of::<nlmsghdr>() + size_of::<ifinfomsg>();

        while offset + 4 <= data.len() {
            let rta_len = unsafe { u16::from_le_bytes(*(data.as_ptr().add(offset) as *const [u8; 2])) };
            let rta_type = unsafe { u16::from_le_bytes(*(data.as_ptr().add(offset + 2) as *const [u8; 2])) };

            if rta_len as usize <= offset {
                break;
            }

            let payload = &data[offset + 4..offset + rta_len as usize];

            match rta_type {
                IFLA_IFNAME => {
                    if let Ok(name) = std::str::from_utf8(payload) {
                        info.name = name.trim_end_matches('\0').to_string();
                    }
                }
                IFLA_ADDRESS => {
                    info.mac_address = payload.to_vec();
                }
                IFLA_MTU => {
                    if payload.len() >= 4 {
                        info.mtu = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
                    }
                }
                _ => {}
            }

            offset = (offset + rta_len as usize + 3) & !3;
        }

        Ok(info)
    }

    pub fn get_addresses(&mut self) -> Result<Vec<AddressInfo>> {
        let mut addresses = Vec::new();

        let hdr = nlmsghdr {
            nlmsg_len: 0,
            nlmsg_type: RTM_GETADDR,
            nlmsg_flags: (libc::NLM_F_REQUEST | libc::NLM_F_DUMP) as u16,
            nlmsg_seq: 0,
            nlmsg_pid: 0,
        };

        let ifam = ifaddrmsg {
            ifa_family: libc::AF_UNSPEC as u8,
            ifa_prefixlen: 0,
            ifa_flags: 0,
            ifa_scope: 0,
            ifa_index: 0,
        };

        let payload = unsafe {
            std::slice::from_raw_parts(&ifam as *const ifaddrmsg as *const u8, size_of::<ifaddrmsg>())
        };

        self.send(hdr, payload)?;

        loop {
            let buf = self.recv()?;

            let mut offset = 0;
            while offset < buf.len() {
                let nlh = unsafe {
                    &*(buf.as_ptr().add(offset) as *const nlmsghdr)
                };

                if nlh.nlmsg_type == libc::NLMSG_DONE as u16 {
                    return Ok(addresses);
                }

                if nlh.nlmsg_type == RTM_GETADDR {
                    // Parse address (simplified)
                }

                offset += nlh.nlmsg_len as usize;
            }
        }
    }

    pub fn get_routes(&mut self) -> Result<Vec<RouteInfo>> {
        let mut routes = Vec::new();

        let hdr = nlmsghdr {
            nlmsg_len: 0,
            nlmsg_type: RTM_GETROUTE,
            nlmsg_flags: (libc::NLM_F_REQUEST | libc::NLM_F_DUMP) as u16,
            nlmsg_seq: 0,
            nlmsg_pid: 0,
        };

        let rtm = rtmsg {
            rtm_family: libc::AF_UNSPEC as u8,
            rtm_dst_len: 0,
            rtm_src_len: 0,
            rtm_tos: 0,
            rtm_table: 0,
            rtm_protocol: 0,
            rtm_scope: 0,
            rtm_type: 0,
            rtm_flags: 0,
        };

        let payload = unsafe {
            std::slice::from_raw_parts(&rtm as *const rtmsg as *const u8, size_of::<rtmsg>())
        };

        self.send(hdr, payload)?;

        loop {
            let buf = self.recv()?;

            let mut offset = 0;
            while offset < buf.len() {
                let nlh = unsafe {
                    &*(buf.as_ptr().add(offset) as *const nlmsghdr)
                };

                if nlh.nlmsg_type == libc::NLMSG_DONE as u16 {
                    return Ok(routes);
                }

                if nlh.nlmsg_type == RTM_GETROUTE {
                    // Parse route (simplified)
                }

                offset += nlh.nlmsg_len as usize;
            }
        }
    }
}

impl Drop for NetlinkSocket {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}
