use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use anyhow::Result;
use pcap_file::{
    DataLink,
    pcap::{PcapHeader, PcapPacket, PcapReader, PcapWriter},
};

use crate::{flows::extract_flow_key, packet::Packet};

pub fn read_pcap(path: impl AsRef<Path>) -> Result<Vec<Packet>> {
    let file = BufReader::new(File::open(path)?);
    let mut reader = PcapReader::new(file)?;

    let mut packets = Vec::new();

    while let Some(packet) = reader.next_packet() {
        let packet = packet?;

        let flow_key = extract_flow_key(&packet.data);

        packets.push(Packet {
            timestamp: packet.timestamp,
            data: packet.data.into(),
            flow_key,
        });
    }

    Ok(packets)
}

pub fn write_pcap(path: impl AsRef<Path>, packets: &[Packet]) -> Result<()> {
    let file = BufWriter::new(File::create(path)?);

    let header = PcapHeader {
        datalink: DataLink::ETHERNET,
        ..Default::default()
    };

    let mut writer = PcapWriter::with_header(file, header)?;

    for packet in packets {
        writer.write_packet(&PcapPacket::new(
            packet.timestamp,
            packet.data.len() as u32,
            &packet.data,
        ))?;
    }

    Ok(())
}
