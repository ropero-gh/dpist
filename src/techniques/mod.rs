use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::packet::Packet;

pub mod delays;
pub mod drop_packets;
pub mod http_fragmentation;
pub mod tcp_fragmentation;
pub mod tcp_out_of_order;
pub mod tls_fragmentation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifierEntry {
    pub enabled: bool,
    pub config: ModifierConfig,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ModifierConfig {
    DropEveryNth { n: u64 },
    Delay(DelayConfig),
    TcpSegmentation { segment_size: usize },

    TlsClientHelloFragmentation { fragment_size: usize },
    TcpOutOfOrder { window: usize },

    HttpHeaderFragmentation { fragment_size: usize },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DelayConfig {
    Fixed { millis: u64 },

    Jitter { min_ms: u64, max_ms: u64 },

    PacketPacing { millis: u64 },

    FlowRateLimit { bytes_per_second: u64 },

    Burst { active_ms: u64, pause_ms: u64 },
}

pub trait Modifier {
    fn modify(&mut self, packet: Packet, output: &mut Vec<Packet>);
}

pub fn build_modifier(config: ModifierConfig) -> Box<dyn Modifier> {
    match config {
        ModifierConfig::DropEveryNth { n } => {
            Box::new(drop_packets::DropEveryNth { n, counter: 0 })
        }

        ModifierConfig::Delay(delay_cfg) => match delay_cfg {
            DelayConfig::Fixed { millis } => Box::new(delays::FixedDelay {
                delay: Duration::from_millis(millis),
            }),

            DelayConfig::Jitter { min_ms, max_ms } => Box::new(delays::JitterDelay::new(
                Duration::from_millis(min_ms),
                Duration::from_millis(max_ms),
            )),

            DelayConfig::PacketPacing { millis } => Box::new(delays::PacketPacingDelay::new(
                Duration::from_millis(millis),
            )),

            DelayConfig::Burst {
                active_ms,
                pause_ms,
            } => Box::new(delays::BurstDelay::new(
                Duration::from_millis(active_ms),
                Duration::from_millis(pause_ms),
            )),
            DelayConfig::FlowRateLimit { bytes_per_second } => {
                Box::new(delays::FlowRateLimitDelay::new(bytes_per_second))
            }
        },
        ModifierConfig::TcpSegmentation { segment_size } => {
            Box::new(tcp_fragmentation::TcpSegmentation::new(segment_size))
        }

        ModifierConfig::TlsClientHelloFragmentation { fragment_size } => Box::new(
            tls_fragmentation::TlsClientHelloFragmentation::new(fragment_size),
        ),
        ModifierConfig::TcpOutOfOrder { window } => {
            Box::new(tcp_out_of_order::TcpOutOfOrder::new(window))
        }

        ModifierConfig::HttpHeaderFragmentation { fragment_size } => Box::new(
            http_fragmentation::HttpHeaderFragmentation::new(fragment_size),
        ),
    }
}

pub fn run_pipeline(packets: Vec<Packet>, modifiers: &mut [Box<dyn Modifier>]) -> Vec<Packet> {
    let mut current = packets;

    for modifier in modifiers {
        let mut next = Vec::with_capacity(current.len());

        for packet in current {
            modifier.modify(packet, &mut next);
        }

        current = next;
    }

    current.sort_unstable_by_key(|p| p.timestamp);

    current
}
