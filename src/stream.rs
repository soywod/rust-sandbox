use std::collections::VecDeque;

use bytes::Bytes;

#[derive(Clone, Debug)]
pub enum StreamEffect {
    Connect(String, u16),
    Disconnect,
    ReadToString,
    WriteAll(Bytes),
}

#[derive(Clone, Debug, Default)]
pub struct StreamState {
    effects: VecDeque<StreamEffect>,
}

impl StreamState {
    pub fn connect(&mut self, host: impl ToString, port: u16) {
        self.effects
            .push_back(StreamEffect::Connect(host.to_string(), port));
    }

    pub fn disconnect(&mut self) {
        self.effects.push_back(StreamEffect::Disconnect);
    }

    pub fn read_to_string(&mut self) {
        self.effects.push_back(StreamEffect::ReadToString);
    }

    pub fn write_all(&mut self, buf: impl Into<Bytes>) {
        self.effects.push_back(StreamEffect::WriteAll(buf.into()));
    }
}

impl Iterator for StreamState {
    type Item = StreamEffect;

    fn next(&mut self) -> Option<Self::Item> {
        println!("stream: next");

        let effect = self.effects.pop_front();
        println!("stream: emit effect {effect:?}");

        effect
    }
}
