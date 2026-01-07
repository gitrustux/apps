// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Raw socket implementation

use anyhow::{Context, Result};
use std::mem::size_of;
use std::os::fd::AsRawFd;
use std::time::Duration;

pub struct RawSocket {
    fd: libc::c_int,
    protocol: i32,
}

impl RawSocket {
    pub fn new(protocol: i32) -> Result<Self> {
        let fd = unsafe {
            libc::socket(
                libc::AF_INET,
                libc::SOCK_RAW | libc::SOCK_CLOEXEC,
                protocol,
            )
        };

        if fd < 0 {
            return Err(anyhow::anyhow!("failed to create raw socket (may require CAP_NET_RAW)"));
        }

        // Set receive timeout
        let timeout = libc::timeval {
            tv_sec: 5,
            tv_usec: 0,
        };

        unsafe {
            libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_RCVTIMEO,
                &timeout as *const _ as *const libc::c_void,
                size_of::<libc::timeval>() as u32,
            );
        }

        Ok(Self { fd, protocol })
    }

    pub fn send_to(&self, buf: &[u8], dest_addr: &std::net::SocketAddr) -> Result<usize> {
        let (addr, addr_len) = match dest_addr {
            std::net::SocketAddr::V4(addr) => {
                let sockaddr = libc::sockaddr_in {
                    sin_family: libc::AF_INET as u16,
                    sin_port: addr.port().to_be(),
                    sin_addr: libc::in_addr {
                        s_addr: u32::from_ne_bytes(addr.ip().octets()),
                    },
                    sin_zero: [0; 8],
                };
                (sockaddr, size_of::<libc::sockaddr_in>())
            }
            std::net::SocketAddr::V6(_) => {
                return Err(anyhow::anyhow!("IPv6 not supported yet"));
            }
        };

        let sent = unsafe {
            libc::sendto(
                self.fd,
                buf.as_ptr() as *const libc::c_void,
                buf.len(),
                0,
                &addr as *const _ as *const libc::sockaddr,
                addr_len as u32,
            )
        };

        if sent < 0 {
            return Err(anyhow::anyhow!("sendto failed"));
        }

        Ok(sent as usize)
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, std::net::SocketAddr)> {
        let mut sockaddr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
        let mut addr_len = size_of::<libc::sockaddr_in>() as u32;

        let recv_len = unsafe {
            libc::recvfrom(
                self.fd,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
                0,
                &mut sockaddr as *mut _ as *mut libc::sockaddr,
                &mut addr_len,
            )
        };

        if recv_len < 0 {
            return Err(anyhow::anyhow!("recvfrom failed"));
        }

        let addr = std::net::SocketAddr::V4(std::net::SocketAddrV4::new(
            std::net::Ipv4Addr::from(unsafe { sockaddr.sin_addr.s_addr }.to_ne_bytes()),
            u16::from_be(unsafe { sockaddr.sin_port }),
        ));

        Ok((recv_len as usize, addr))
    }

    pub fn set_timeout(&self, duration: Duration) -> Result<()> {
        let timeout = libc::timeval {
            tv_sec: duration.as_secs() as i64,
            tv_usec: duration.subsec_micros() as i64,
        };

        unsafe {
            if libc::setsockopt(
                self.fd,
                libc::SOL_SOCKET,
                libc::SO_RCVTIMEO,
                &timeout as *const _ as *const libc::c_void,
                size_of::<libc::timeval>() as u32,
            ) < 0
            {
                return Err(anyhow::anyhow!("failed to set timeout"));
            }
        }

        Ok(())
    }
}

impl Drop for RawSocket {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}

pub struct IcmpSocket {
    raw: RawSocket,
}

impl IcmpSocket {
    pub fn new() -> Result<Self> {
        Ok(Self {
            raw: RawSocket::new(libc::IPPROTO_ICMP)?,
        })
    }

    pub fn send_to(&self, buf: &[u8], dest: &std::net::Ipv4Addr) -> Result<usize> {
        let addr = std::net::SocketAddr::V4(std::net::SocketAddrV4::new(*dest, 0));
        self.raw.send_to(buf, &addr)
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, std::net::SocketAddr)> {
        self.raw.recv_from(buf)
    }

    pub fn set_timeout(&self, duration: Duration) -> Result<()> {
        self.raw.set_timeout(duration)
    }
}
