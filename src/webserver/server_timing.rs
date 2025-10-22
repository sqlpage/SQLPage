use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ServerTiming {
    start: Instant,
    events: Vec<TimingEvent>,
}

#[derive(Debug, Clone)]
struct TimingEvent {
    name: &'static str,
    duration_ms: f64,
}

impl ServerTiming {
    #[must_use]
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            events: Vec::new(),
        }
    }

    pub fn record(&mut self, name: &'static str) {
        let duration_ms = self.start.elapsed().as_secs_f64() * 1000.0;
        self.events.push(TimingEvent { name, duration_ms });
    }

    #[must_use]
    pub fn as_header_value(&self) -> String {
        self.events
            .iter()
            .map(|event| format!("{};dur={:.2}", event.name, event.duration_ms))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl Default for ServerTiming {
    fn default() -> Self {
        Self::new()
    }
}
