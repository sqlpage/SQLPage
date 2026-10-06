//! Native service logs, including failures before the tracing subscriber exists.

use super::background_log::BackgroundLog;
use std::{io, sync::OnceLock};
use windows_sys::Win32::{
    Foundation::HANDLE,
    System::EventLog::{
        DeregisterEventSource, EVENTLOG_ERROR_TYPE, EVENTLOG_INFORMATION_TYPE,
        EVENTLOG_WARNING_TYPE, RegisterEventSourceW, ReportEventW,
    },
};

static LOGGER: OnceLock<Result<BackgroundLog, String>> = OnceLock::new();

/// Opens the event source on a dedicated writer thread before serving requests.
pub(super) fn init() -> anyhow::Result<()> {
    logger().map(|_| ()).map_err(anyhow::Error::msg)
}

fn logger() -> Result<&'static BackgroundLog, &'static str> {
    LOGGER
        .get_or_init(|| {
            BackgroundLog::start(|| {
                let source = EventSource::open()?;
                Ok(move |level, message: &str| source.write(level, message))
            })
            .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(String::as_str)
}

/// Queues one event without waiting for the Windows Event Log service.
/// Overload drops records with a warning summary; shutdown flushes accepted records.
pub fn write(level: tracing::Level, message: String) {
    match logger() {
        Ok(logger) => logger.write(level, message),
        Err(error) => eprintln!("Unable to initialize Windows service logging: {error}: {message}"),
    }
}

/// Flushes queued records and releases the event source after producers have stopped.
/// This can block and must be called outside HTTP worker threads.
pub fn shutdown() {
    if let Some(Ok(logger)) = LOGGER.get() {
        logger.shutdown();
    }
}

// Created, used, and dropped exclusively on the log writer thread.
struct EventSource(HANDLE);

impl EventSource {
    fn open() -> io::Result<Self> {
        // SAFETY: Both arguments are valid constant pointers. A successful handle
        // is owned by EventSource and released in Drop.
        let handle = unsafe { RegisterEventSourceW(std::ptr::null(), windows_sys::w!("SQLPage")) };
        if handle.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(Self(handle))
        }
    }

    fn write(&self, level: tracing::Level, message: &str) {
        let event_type = match level {
            tracing::Level::ERROR => EVENTLOG_ERROR_TYPE,
            tracing::Level::WARN => EVENTLOG_WARNING_TYPE,
            _ => EVENTLOG_INFORMATION_TYPE,
        };
        // Queued messages are bounded to 16 KiB, below ReportEvent's limit.
        let message: Vec<u16> = message
            .encode_utf16()
            .map(|unit| if unit == 0 { 0xfffd } else { unit })
            .chain(Some(0))
            .collect();
        let strings = [message.as_ptr()];
        // SAFETY: The event source remains live and all strings are NUL-terminated
        // and valid for the duration of this synchronous call.
        unsafe {
            ReportEventW(
                self.0,
                event_type,
                0,
                0,
                std::ptr::null_mut(),
                1,
                0,
                strings.as_ptr(),
                std::ptr::null(),
            );
        }
    }
}

impl Drop for EventSource {
    fn drop(&mut self) {
        // SAFETY: This is the sole owner of the handle returned by registration.
        unsafe {
            DeregisterEventSource(self.0);
        }
    }
}
