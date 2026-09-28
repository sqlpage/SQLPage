//! Native service logs, including failures before the tracing subscriber exists.

use windows_sys::Win32::System::EventLog::{
    DeregisterEventSource, EVENTLOG_ERROR_TYPE, EVENTLOG_INFORMATION_TYPE, EVENTLOG_WARNING_TYPE,
    RegisterEventSourceW, ReportEventW,
};

/// Writes one event under the `SQLPage` source in the Application log.
pub fn write(level: tracing::Level, message: &str) {
    let event_type = match level {
        tracing::Level::ERROR => EVENTLOG_ERROR_TYPE,
        tracing::Level::WARN => EVENTLOG_WARNING_TYPE,
        _ => EVENTLOG_INFORMATION_TYPE,
    };
    // ReportEvent limits insertion strings to 31,839 UTF-16 code units.
    let message: Vec<u16> = message
        .encode_utf16()
        .take(31_838)
        .map(|unit| if unit == 0 { 0xfffd } else { unit })
        .chain(Some(0))
        .collect();
    let strings = [message.as_ptr()];
    // SAFETY: All pointers refer to live, NUL-terminated strings. The returned
    // handle is used only here and is released after the synchronous write.
    unsafe {
        let handle = RegisterEventSourceW(std::ptr::null(), windows_sys::w!("SQLPage"));
        if handle.is_null() {
            return;
        }
        ReportEventW(
            handle,
            event_type,
            0,
            0,
            std::ptr::null(),
            1,
            0,
            strings.as_ptr(),
            std::ptr::null(),
        );
        DeregisterEventSource(handle);
    }
}
