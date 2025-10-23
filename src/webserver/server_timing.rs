use std::fmt::Write;
use std::sync::Mutex;
use std::time::Instant;

use crate::app_config::DevOrProd;

#[derive(Debug)]
pub struct ServerTiming {
    enabled: bool,
    created_at: Instant,
    events: Mutex<Vec<PerfEvent>>,
}

#[derive(Debug)]
struct PerfEvent {
    time: Instant,
    name: &'static str,
}

impl Default for ServerTiming {
    fn default() -> Self {
        Self {
            enabled: false,
            created_at: Instant::now(),
            events: Mutex::new(Vec::new()),
        }
    }
}

impl ServerTiming {
    #[must_use]
    pub fn enabled(enabled: bool) -> Self {
        Self {
            enabled,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn for_env(env: DevOrProd) -> Self {
        Self::enabled(!env.is_prod())
    }

    pub fn record(&self, name: &'static str) {
        if self.enabled {
            self.events.lock().unwrap().push(PerfEvent {
                time: Instant::now(),
                name,
            });
        }
    }

    pub fn header_value(&self) -> Option<String> {
        if !self.enabled {
            return None;
        }
        let evts = self.events.lock().unwrap();
        let mut s = String::with_capacity(evts.len() * 16);
        let mut last = self.created_at;
        for (i, PerfEvent { name, time }) in evts.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            let millis = time.saturating_duration_since(last).as_millis();
            write!(&mut s, "{name};dur={millis}").ok()?;
            last = *time;
        }
        Some(s)
    }
}
