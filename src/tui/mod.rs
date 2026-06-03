pub mod app;
pub mod input;
pub mod ui;
pub mod ui_schema;

use anyhow::Result;

use crate::{
    config::{self, ModifierUi},
    techniques::{DelayConfig, ModifierConfig},
};

pub fn modifier_name(cfg: &ModifierConfig) -> &'static str {
    match cfg {
        ModifierConfig::Delay(DelayConfig::Fixed { .. }) => "Delay (Fixed)",
        ModifierConfig::Delay(DelayConfig::Jitter { .. }) => "Delay (Jitter)",
        ModifierConfig::Delay(DelayConfig::PacketPacing { .. }) => "Delay (Packet Pacing)",
        ModifierConfig::Delay(DelayConfig::FlowRateLimit { .. }) => "Delay (Rate Limit)",
        ModifierConfig::Delay(DelayConfig::Burst { .. }) => "Delay (Burst)",
        ModifierConfig::TcpSegmentation { .. } => "TCP Segmentation",
        ModifierConfig::TlsClientHelloFragmentation { .. } => "TLS ClientHello Fragmentation",
        ModifierConfig::DropEveryNth { .. } => "Drop Every Nth",
        ModifierConfig::TcpOutOfOrder { .. } => "TCP Out-of-Order",
        ModifierConfig::HttpHeaderFragmentation { .. } => "HTTP Header Fragmentation",
    }
}

fn modifier_key(cfg: &ModifierConfig) -> &'static str {
    match cfg {
        ModifierConfig::Delay(DelayConfig::Fixed { .. }) => "delay_fixed",
        ModifierConfig::Delay(DelayConfig::Jitter { .. }) => "delay_jitter",
        ModifierConfig::Delay(DelayConfig::PacketPacing { .. }) => "delay_packet_pacing",
        ModifierConfig::Delay(DelayConfig::FlowRateLimit { .. }) => "delay_rate_limit",
        ModifierConfig::Delay(DelayConfig::Burst { .. }) => "delay_burst",
        ModifierConfig::TcpSegmentation { .. } => "tcp_segmentation",
        ModifierConfig::TlsClientHelloFragmentation { .. } => "tls_clienthello_fragmentation",
        ModifierConfig::DropEveryNth { .. } => "drop_every_nth",
        ModifierConfig::TcpOutOfOrder { .. } => "tcp_out_of_order",
        ModifierConfig::HttpHeaderFragmentation { .. } => "http_header_fragmentation",
    }
}

fn default_modifiers() -> Vec<ModifierUi> {
    vec![
        ModifierUi {
            enabled: false,
            config: ModifierConfig::Delay(DelayConfig::Fixed { millis: 100 }),
        },
        ModifierUi {
            enabled: false,
            config: ModifierConfig::Delay(DelayConfig::Jitter {
                min_ms: 50,
                max_ms: 200,
            }),
        },
        ModifierUi {
            enabled: false,
            config: ModifierConfig::Delay(DelayConfig::PacketPacing { millis: 100 }),
        },
        ModifierUi {
            enabled: false,
            config: ModifierConfig::Delay(DelayConfig::FlowRateLimit {
                bytes_per_second: 100_000,
            }),
        },
        ModifierUi {
            enabled: false,
            config: ModifierConfig::Delay(DelayConfig::Burst {
                active_ms: 1000,
                pause_ms: 500,
            }),
        },
        ModifierUi {
            enabled: false,
            config: ModifierConfig::TcpSegmentation { segment_size: 128 },
        },
        ModifierUi {
            enabled: false,
            config: ModifierConfig::TlsClientHelloFragmentation { fragment_size: 64 },
        },
        ModifierUi {
            enabled: false,
            config: ModifierConfig::DropEveryNth { n: 10 },
        },
    ]
}

fn merge_modifiers(loaded: Vec<ModifierConfig>) -> Vec<ModifierUi> {
    let mut loaded_ui: Vec<ModifierUi> = loaded
        .into_iter()
        .map(|c| ModifierUi {
            enabled: true,
            config: c,
        })
        .collect();

    for default in default_modifiers() {
        let key = modifier_key(&default.config);

        let exists = loaded_ui.iter().any(|m| modifier_key(&m.config) == key);

        if !exists {
            loaded_ui.push(default);
        }
    }

    loaded_ui
}

pub fn run(path: &str) -> Result<()> {
    let config = config::load_config(path)?;

    let modifier_ui = merge_modifiers(config.modifiers.clone());

    let mut app = app::App::new(config, modifier_ui, "config.toml");

    ui::run_tui(&mut app)
}
