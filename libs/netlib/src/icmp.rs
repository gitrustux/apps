// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! ICMP packet implementation

use std::time::Instant;

pub const ICMP_ECHO_REQUEST: u8 = 8;
pub const ICMP_ECHO_REPLY: u8 = 0;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct IcmpHeader {
    pub icmp_type: u8,
    pub icmp_code: u8,
    pub icmp_cksum: u16,
    pub icmp_id: u16,
    pub icmp_seq: u16,
}

#[derive(Debug, Clone)]
pub struct IcmpPacket {
    pub header: IcmpHeader,
    pub payload: Vec<u8>,
}

impl IcmpPacket {
    pub fn new_echo_request(id: u16, seq: u16, payload: &[u8]) -> Self {
        let mut header = IcmpHeader {
            icmp_type: ICMP_ECHO_REQUEST,
            icmp_code: 0,
            icmp_cksum: 0,
            icmp_id: id,
            icmp_seq: seq,
        };

        let mut packet = Self {
            header,
            payload: payload.to_vec(),
        };

        packet.header.icmp_cksum = packet.compute_checksum();
        packet
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }

        let header = IcmpHeader {
            icmp_type: data[0],
            icmp_code: data[1],
            icmp_cksum: u16::from_be_bytes([data[2], data[3]]),
            icmp_id: u16::from_be_bytes([data[4], data[5]]),
            icmp_seq: u16::from_be_bytes([data[6], data[7]]),
        };

        let payload = data[8..].to_vec();

        Some(Self { header, payload })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(8 + self.payload.len());

        buf.push(self.header.icmp_type);
        buf.push(self.header.icmp_code);
        buf.extend_from_slice(&self.header.icmp_cksum.to_be_bytes());
        buf.extend_from_slice(&self.header.icmp_id.to_be_bytes());
        buf.extend_from_slice(&self.header.icmp_seq.to_be_bytes());
        buf.extend_from_slice(&self.payload);

        buf
    }

    fn compute_checksum(&self) -> u16 {
        let mut sum: u32 = 0;

        // Add header words
        sum += self.header.icmp_type as u32;
        sum += self.header.icmp_code as u32;
        sum += self.header.icmp_id as u32;
        sum += self.header.icmp_seq as u32;

        // Add payload
        let mut i = 0;
        while i + 1 < self.payload.len() {
            let word = u16::from_be_bytes([self.payload[i], self.payload[i + 1]]) as u32;
            sum += word;
            i += 2;
        }

        // Handle odd byte
        if i < self.payload.len() {
            sum += (self.payload[i] as u32) << 8;
        }

        // Fold 32-bit sum to 16 bits
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        !sum as u16
    }

    pub fn is_valid(&self) -> bool {
        self.compute_checksum() == self.header.icmp_cksum
    }

    pub fn is_echo_reply(&self) -> bool {
        self.header.icmp_type == ICMP_ECHO_REPLY
    }
}

#[derive(Debug, Clone)]
pub struct IcmpEchoReply {
    pub id: u16,
    pub seq: u16,
    pub payload: Vec<u8>,
    pub rtt_ms: f64,
}

pub struct PingStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub total_time_ms: f64,
    pub min_rtt_ms: f64,
    pub max_rtt_ms: f64,
}

impl PingStats {
    pub fn new() -> Self {
        Self {
            packets_sent: 0,
            packets_received: 0,
            total_time_ms: 0.0,
            min_rtt_ms: f64::MAX,
            max_rtt_ms: 0.0,
        }
    }

    pub fn update(&mut self, rtt_ms: f64) {
        self.packets_sent += 1;
        self.packets_received += 1;
        self.total_time_ms += rtt_ms;
        self.min_rtt_ms = self.min_rtt_ms.min(rtt_ms);
        self.max_rtt_ms = self.max_rtt_ms.max(rtt_ms);
    }

    pub fn packet_loss_percent(&self) -> f64 {
        if self.packets_sent == 0 {
            return 0.0;
        }
        ((self.packets_sent - self.packets_received) as f64 / self.packets_sent as f64) * 100.0
    }

    pub fn avg_rtt_ms(&self) -> f64 {
        if self.packets_received == 0 {
            return 0.0;
        }
        self.total_time_ms / self.packets_received as f64
    }
}
