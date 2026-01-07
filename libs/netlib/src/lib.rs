// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! netlib - Networking Library for Rustica
//!
//! Provides netlink socket, raw socket, and protocol implementations.

pub mod netlink;
pub mod raw;
pub mod icmp;

pub use netlink::{NetlinkSocket, InterfaceInfo};
pub use raw::{RawSocket, IcmpSocket};
pub use icmp::{IcmpPacket, IcmpEchoReply, PingStats, ICMP_ECHO_REQUEST, ICMP_ECHO_REPLY};
