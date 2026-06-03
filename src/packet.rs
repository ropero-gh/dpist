use std::time::Duration;

use crate::flows::FlowKey;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Packet {
    pub timestamp: Duration,
    pub data: Vec<u8>,
    pub flow_key: Option<FlowKey>,
}
