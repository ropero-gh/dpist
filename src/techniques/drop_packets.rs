use crate::{packet::Packet, techniques::Modifier};

pub struct DropEveryNth {
    pub n: u64,
    pub counter: u64,
}

impl Modifier for DropEveryNth {
    fn modify(&mut self, packet: Packet, output: &mut Vec<Packet>) {
        self.counter += 1;

        if !self.counter.is_multiple_of(self.n) {
            output.push(packet);
        }
    }
}
