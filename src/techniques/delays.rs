use std::{collections::HashMap, time::Duration};

use rand::{RngExt, SeedableRng, rngs::StdRng};

use crate::{
    flows::{FlowKey, extract_flow_key},
    packet::Packet,
    techniques::Modifier,
};

pub struct FixedDelay {
    pub delay: Duration,
}

impl Modifier for FixedDelay {
    fn modify(&mut self, mut packet: Packet, output: &mut Vec<Packet>) {
        packet.timestamp += self.delay;
        output.push(packet);
    }
}

pub struct JitterDelay {
    pub min: Duration,
    pub max: Duration,
    rng: StdRng,
}

impl JitterDelay {
    pub fn new(min: Duration, max: Duration) -> Self {
        Self {
            min,
            max,
            rng: StdRng::seed_from_u64(42),
        }
    }

    fn sample(&mut self) -> Duration {
        let min_ms = self.min.as_millis() as u64;
        let max_ms = self.max.as_millis() as u64;

        Duration::from_millis(self.rng.random_range(min_ms..=max_ms))
    }
}

impl Modifier for JitterDelay {
    fn modify(&mut self, mut packet: Packet, output: &mut Vec<Packet>) {
        packet.timestamp += self.sample();
        output.push(packet);
    }
}
pub struct PacketPacingDelay {
    pub delay_per_packet: Duration,

    next_departure: HashMap<FlowKey, Duration>,
}

impl PacketPacingDelay {
    pub fn new(delay_per_packet: Duration) -> Self {
        Self {
            delay_per_packet,
            next_departure: HashMap::new(),
        }
    }
}

impl Modifier for PacketPacingDelay {
    fn modify(&mut self, mut packet: Packet, output: &mut Vec<Packet>) {
        let Some(flow) = extract_flow_key(&packet.data) else {
            output.push(packet);
            return;
        };

        let next = self.next_departure.entry(flow).or_insert(packet.timestamp);

        let departure = (*next).max(packet.timestamp);

        *next = departure + self.delay_per_packet;

        packet.timestamp = departure;
        output.push(packet);
    }
}
pub struct BurstDelay {
    pub active: Duration,
    pub pause: Duration,
}

impl BurstDelay {
    pub fn new(active: Duration, pause: Duration) -> Self {
        Self { active, pause }
    }
}

impl Modifier for BurstDelay {
    fn modify(&mut self, mut packet: Packet, output: &mut Vec<Packet>) {
        let cycle = self.active + self.pause;

        let cycle_ms = cycle.as_millis() as u64;
        let packet_ms = packet.timestamp.as_millis() as u64;

        let pos = Duration::from_millis(packet_ms % cycle_ms);

        if pos >= self.active {
            let wait = cycle - pos;
            packet.timestamp += wait;
        }

        output.push(packet);
    }
}

pub struct FlowRateLimitDelay {
    pub bytes_per_second: u64,

    next_departure: HashMap<FlowKey, Duration>,
}

impl FlowRateLimitDelay {
    pub fn new(bytes_per_second: u64) -> Self {
        Self {
            bytes_per_second,
            next_departure: HashMap::new(),
        }
    }
}

impl Modifier for FlowRateLimitDelay {
    fn modify(&mut self, mut packet: Packet, output: &mut Vec<Packet>) {
        let Some(flow) = extract_flow_key(&packet.data) else {
            output.push(packet);
            return;
        };

        let transmission_time =
            Duration::from_secs_f64(packet.data.len() as f64 / self.bytes_per_second as f64);

        let next = self.next_departure.entry(flow).or_insert(packet.timestamp);

        let departure = (*next).max(packet.timestamp);

        *next = departure + transmission_time;

        packet.timestamp = departure;
        output.push(packet);
    }
}
