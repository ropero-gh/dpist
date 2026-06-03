use crate::{packet::Packet, techniques::Modifier};

#[derive(Debug, Clone)]
pub struct HttpHeaderFragmentation {
    pub fragment_size: usize,
}

impl HttpHeaderFragmentation {
    pub fn new(fragment_size: usize) -> Self {
        Self { fragment_size }
    }

    fn is_http_request(data: &[u8]) -> bool {
        data.starts_with(b"GET ")
            || data.starts_with(b"POST ")
            || data.starts_with(b"PUT ")
            || data.starts_with(b"DELETE ")
            || data.starts_with(b"HEAD ")
    }
}

impl Modifier for HttpHeaderFragmentation {
    fn modify(&mut self, packet: Packet, output: &mut Vec<Packet>) {
        let data = packet.data.clone();

        if !Self::is_http_request(&data) {
            output.push(packet);
            return;
        }

        let mut offset = 0usize;

        for chunk in data.chunks(self.fragment_size) {
            let mut new_packet = packet.clone();
            new_packet.data = chunk.to_vec();

            new_packet.timestamp += std::time::Duration::from_micros(offset as u64);
            output.push(new_packet);

            offset += 1;
        }
    }
}
