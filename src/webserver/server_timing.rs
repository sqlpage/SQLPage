use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ServerTiming {
    enabled: bool,
    previous_event: Instant,
    header: String,
}

impl ServerTiming {
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            previous_event: Instant::now(),
            header: String::new(),
        }
    }

    pub fn record(&mut self, name: &'static str) {
        if !self.enabled {
            return;
        }
        let now = Instant::now();
        let duration_ms = (now - self.previous_event).as_secs_f64() * 1000.0;
        self.previous_event = now;

        if !self.header.is_empty() {
            self.header.push_str(", ");
        }
        use std::fmt::Write;
        write!(&mut self.header, "{};dur={:.2}", name, duration_ms).unwrap();
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[must_use]
    pub fn as_header_value(&self) -> &str {
        if self.enabled {
            &self.header
        } else {
            ""
        }
    }
}

impl Default for ServerTiming {
    fn default() -> Self {
        Self::new(false)
    }
}
