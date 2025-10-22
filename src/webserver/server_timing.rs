use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ServerTiming {
    enabled: bool,
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
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            start: Instant::now(),
            events: Vec::new(),
        }
    }

    pub fn record(&mut self, name: &'static str) {
        if !self.enabled {
            return;
        }
        let duration_ms = self.start.elapsed().as_secs_f64() * 1000.0;
        self.events.push(TimingEvent { name, duration_ms });
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[must_use]
    pub fn as_header_value(&self) -> String {
        if !self.enabled {
            return String::new();
        }
        self.events
            .iter()
            .map(|event| format!("{};dur={:.2}", event.name, event.duration_ms))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl Default for ServerTiming {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_disabled_timing() {
        let mut timing = ServerTiming::new(false);
        assert!(!timing.is_enabled());
        timing.record("event1");
        timing.record("event2");
        assert_eq!(timing.as_header_value(), "");
    }

    #[test]
    fn test_enabled_timing() {
        let mut timing = ServerTiming::new(true);
        assert!(timing.is_enabled());
        timing.record("event1");
        thread::sleep(Duration::from_millis(10));
        timing.record("event2");
        let header = timing.as_header_value();
        assert!(header.contains("event1;dur="));
        assert!(header.contains("event2;dur="));
        assert!(header.contains(", "));
    }

    #[test]
    fn test_timing_values_increase() {
        let mut timing = ServerTiming::new(true);
        timing.record("first");
        thread::sleep(Duration::from_millis(5));
        timing.record("second");
        assert_eq!(timing.events.len(), 2);
        assert!(timing.events[1].duration_ms > timing.events[0].duration_ms);
    }

    #[test]
    fn test_default_is_disabled() {
        let timing = ServerTiming::default();
        assert!(!timing.is_enabled());
    }

    #[test]
    fn test_header_format() {
        let mut timing = ServerTiming::new(true);
        timing.events.push(TimingEvent {
            name: "test",
            duration_ms: 123.456,
        });
        let header = timing.as_header_value();
        assert_eq!(header, "test;dur=123.46");
    }

    #[test]
    fn test_multiple_events_format() {
        let mut timing = ServerTiming::new(true);
        timing.events.push(TimingEvent {
            name: "first",
            duration_ms: 10.5,
        });
        timing.events.push(TimingEvent {
            name: "second",
            duration_ms: 25.75,
        });
        let header = timing.as_header_value();
        assert_eq!(header, "first;dur=10.50, second;dur=25.75");
    }
}
