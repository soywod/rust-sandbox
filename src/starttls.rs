use crate::stream::StreamState;

#[derive(Clone, Debug, Default)]
pub struct StartTlsProvider {
    host: String,
    port: u16,
}

impl StartTlsProvider {
    pub fn new(host: impl ToString, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
        }
    }

    pub fn imap(&self) -> StreamState {
        let mut state = StreamState::default();

        state.connect(&self.host, self.port);
        state.discard_line();

        state.write_line("A1 STARTTLS");
        state.discard_line();

        state.upgrade(self.host.clone());
        state.write_line("A2 CAPABILITY");
        state.read_line();

        state.disconnect();

        state
    }

    pub fn smtp(&self, helo: impl AsRef<str>) -> StreamState {
        let mut state = StreamState::default();

        state.connect(&self.host, self.port);
        state.discard_line();

        state.write_line(format!("HELO {}", helo.as_ref()));
        state.discard_line();

        state.write_line("STARTTLS");
        state.discard_line();

        state.upgrade(self.host.clone());
        state.write_line("NOOP");
        state.discard_line();

        state.disconnect();

        state
    }
}
