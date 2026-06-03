use std::net::IpAddr;

use etherparse::SlicedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FlowKey {
    pub src_ip: IpAddr,
    pub dst_ip: IpAddr,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: u8,
}

impl FlowKey {
    pub fn normalized(mut self) -> Self {
        let left = (self.src_ip, self.src_port);
        let right = (self.dst_ip, self.dst_port);

        if left > right {
            std::mem::swap(&mut self.src_ip, &mut self.dst_ip);
            std::mem::swap(&mut self.src_port, &mut self.dst_port);
        }

        self
    }
}

pub fn extract_flow_key(data: &[u8]) -> Option<FlowKey> {
    let packet = SlicedPacket::from_ethernet(data).ok()?;

    let (src_ip, dst_ip, protocol) = match packet.net? {
        etherparse::NetSlice::Ipv4(ipv4) => (
            IpAddr::V4(ipv4.header().source_addr()),
            IpAddr::V4(ipv4.header().destination_addr()),
            ipv4.header().protocol().0,
        ),
        etherparse::NetSlice::Ipv6(ipv6) => (
            IpAddr::V6(ipv6.header().source_addr()),
            IpAddr::V6(ipv6.header().destination_addr()),
            ipv6.header().next_header().0,
        ),
        _ => return None,
    };

    let (src_port, dst_port) = match packet.transport? {
        etherparse::TransportSlice::Tcp(tcp) => (tcp.source_port(), tcp.destination_port()),
        etherparse::TransportSlice::Udp(udp) => (udp.source_port(), udp.destination_port()),
        _ => return None,
    };

    Some(
        FlowKey {
            src_ip,
            dst_ip,
            src_port,
            dst_port,
            protocol,
        }
        .normalized(),
    )
}
