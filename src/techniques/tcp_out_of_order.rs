use std::collections::HashMap;

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

use crate::packet::Packet;

pub struct TcpOutOfOrder {
    window: usize,
    buffers: HashMap<Option<crate::flows::FlowKey>, Vec<Packet>>,
    rng: StdRng,
}

impl TcpOutOfOrder {
    pub fn new(window: usize) -> Self {
        Self {
            window,
            buffers: HashMap::new(),
            rng: StdRng::seed_from_u64(42),
        }
    }

    fn reorder(&mut self, mut buf: Vec<Packet>, output: &mut Vec<Packet>) {
        buf.reverse();

        for mut pkt in buf {
            let jitter = self.rng.random_range(0..=self.window as u64);
            pkt.timestamp += std::time::Duration::from_millis(jitter);
            output.push(pkt);
        }
    }
}

impl crate::techniques::Modifier for TcpOutOfOrder {
    fn modify(&mut self, packet: Packet, output: &mut Vec<Packet>) {
        let key = packet.flow_key.unwrap_or_else(|| crate::flows::FlowKey {
            src_ip: "0.0.0.0".parse().unwrap(),
            dst_ip: "0.0.0.0".parse().unwrap(),
            src_port: 0,
            dst_port: 0,
            protocol: 0,
        });

        let buf = self.buffers.entry(Some(key.clone())).or_default();
        buf.push(packet.clone());

        if buf.len() >= self.window {
            let buf = self.buffers.remove(&Some(key.clone())).unwrap();
            self.reorder(buf, output);
            self.buffers.insert(Some(key), Vec::new());
        }
    }
}
