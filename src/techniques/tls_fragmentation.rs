use crate::{packet::Packet, techniques::Modifier};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TlsClientHelloFragmentation {
    pub fragment_size: usize,
}

impl TlsClientHelloFragmentation {
    pub fn new(fragment_size: usize) -> Self {
        Self { fragment_size }
    }

    fn is_ipv4_tcp(data: &[u8]) -> bool {
        if data.len() < 34 {
            return false;
        }
        let ethertype = u16::from_be_bytes([data[12], data[13]]);
        ethertype == 0x0800
    }

    fn is_client_hello(payload: &[u8]) -> bool {
        payload.len() > 6 && payload[0] == 0x16 && payload[1] == 0x03 && payload[5] == 0x01
    }
}

impl Modifier for TlsClientHelloFragmentation {
    fn modify(&mut self, packet: Packet, output: &mut Vec<Packet>) {
        let data = packet.data.clone();

        if !Self::is_ipv4_tcp(&data) {
            output.push(packet);
            return;
        }

        let ip_header_len = 14 + 20;
        if data.len() <= ip_header_len {
            output.push(packet);
            return;
        }

        let (header, payload) = data.split_at(ip_header_len);

        if !Self::is_client_hello(payload) {
            output.push(packet);
            return;
        }

        let mut offset = 0u64;

        for chunk in payload.chunks(self.fragment_size) {
            let mut new_packet = packet.clone();
            new_packet.data = {
                let mut v = Vec::with_capacity(header.len() + chunk.len());
                v.extend_from_slice(header);
                v.extend_from_slice(chunk);
                v
            };

            new_packet.timestamp += Duration::from_micros(offset);
            output.push(new_packet);

            offset += 1;
        }
    }
}
