use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub enum SocketEffect {
    Connect(String, u16),
    Disconnect,
}

#[derive(Clone, Debug, Default)]
pub struct SocketState {
    state: InternalState,
    effects: VecDeque<SocketEffect>,
}

impl SocketState {
    pub fn connect(&mut self, host: impl ToString, port: u16) {
        if self.state.can_connect() {
            self.state.set(InternalState::Connected);
            self.effects
                .push_back(SocketEffect::Connect(host.to_string(), port));
        }
    }

    pub fn disconnect(&mut self) {
        if self.state.can_disconnect() {
            self.state.set(InternalState::Disconnected);
            self.effects.push_back(SocketEffect::Disconnect);
        }
    }
}

impl Iterator for SocketState {
    type Item = SocketEffect;

    fn next(&mut self) -> Option<Self::Item> {
        println!("socket: next");

        let effect = self.effects.pop_front();
        println!("socket: emit effect {effect:?}");

        effect
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
enum InternalState {
    Connected,
    #[default]
    Disconnected,
}

impl InternalState {
    pub fn can_connect(&self) -> bool {
        let state = matches!(self, Self::Disconnected);
        println!("socket: can connect? {state:?}");
        state
    }

    pub fn can_disconnect(&self) -> bool {
        let state = matches!(self, Self::Connected);
        println!("socket: can disconnect? {state:?}");
        state
    }

    pub fn set(&mut self, state: Self) {
        if self != &state {
            println!("socket: switch to state {state:?}");
            *self = state;
        }
    }
}
