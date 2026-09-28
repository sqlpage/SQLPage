//! Bounded, non-blocking submission to a blocking platform log writer.

use std::{
    io,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
        mpsc::{RecvTimeoutError, SyncSender, sync_channel},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tracing::Level;

const QUEUE_CAPACITY: usize = 256;
const MAX_MESSAGE_BYTES: usize = 16 * 1024;

enum Message {
    Record(Level, String),
    Shutdown,
}

pub(super) struct BackgroundLog {
    sender: SyncSender<Message>,
    dropped: Arc<AtomicU64>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl BackgroundLog {
    pub(super) fn start<F, W>(make_writer: F) -> io::Result<Self>
    where
        F: FnOnce() -> io::Result<W> + Send + 'static,
        W: FnMut(Level, &str) + 'static,
    {
        let (sender, receiver) = sync_channel(QUEUE_CAPACITY);
        let (started_tx, started_rx) = sync_channel(1);
        let dropped = Arc::new(AtomicU64::new(0));
        let worker_dropped = Arc::clone(&dropped);
        let worker = thread::Builder::new()
            .name("sqlpage-event-log".into())
            .spawn(move || {
                // Construct the writer on its own thread so native handles never
                // need to be shared with HTTP workers.
                let mut writer = match make_writer() {
                    Ok(writer) => writer,
                    Err(error) => {
                        let _ = started_tx.send(Err(error));
                        return;
                    }
                };
                let _ = started_tx.send(Ok(()));
                let mut last_report = Instant::now();
                loop {
                    match receiver.recv_timeout(Duration::from_secs(5)) {
                        Ok(Message::Record(level, message)) => writer(level, &message),
                        Ok(Message::Shutdown) | Err(RecvTimeoutError::Disconnected) => break,
                        Err(RecvTimeoutError::Timeout) => {}
                    }
                    if last_report.elapsed() >= Duration::from_secs(5) {
                        report_dropped(&worker_dropped, &mut writer);
                        last_report = Instant::now();
                    }
                }
                report_dropped(&worker_dropped, &mut writer);
            })?;
        if let Err(error) = started_rx
            .recv()
            .map_err(io::Error::other)
            .and_then(|result| result)
        {
            let _ = worker.join();
            return Err(error);
        }
        Ok(Self {
            sender,
            dropped,
            worker: Mutex::new(Some(worker)),
        })
    }

    pub(super) fn write(&self, level: Level, mut message: String) {
        // Bound both the length and retained allocation, including multibyte text.
        let mut end = MAX_MESSAGE_BYTES.min(message.len());
        while !message.is_char_boundary(end) {
            end -= 1;
        }
        message.truncate(end);
        if message.capacity() > MAX_MESSAGE_BYTES {
            message.shrink_to(MAX_MESSAGE_BYTES);
        }
        if self
            .sender
            .try_send(Message::Record(level, message))
            .is_err()
        {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Call only after producers have stopped. This can block while flushing.
    pub(super) fn shutdown(&self) {
        let worker = self.worker.lock().unwrap().take();
        if let Some(worker) = worker {
            let _ = self.sender.send(Message::Shutdown);
            let _ = worker.join();
        }
    }
}

fn report_dropped(dropped: &AtomicU64, writer: &mut impl FnMut(Level, &str)) {
    let count = dropped.swap(0, Ordering::Relaxed);
    if count != 0 {
        writer(
            Level::WARN,
            &format!("SQLPage dropped {count} log records because the Event Log queue was full"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slow_writer_cannot_block_producers_and_shutdown_flushes_the_queue() {
        let (entered_tx, entered_rx) = sync_channel(1);
        let (release_tx, release_rx) = sync_channel(1);
        let records = Arc::new(Mutex::new(Vec::new()));
        let saved = Arc::clone(&records);
        let logger = Arc::new(
            BackgroundLog::start(move || {
                Ok(move |_level, message: &str| {
                    if message == "first" {
                        entered_tx.send(()).unwrap();
                        release_rx.recv().unwrap();
                    }
                    saved.lock().unwrap().push(message.to_owned());
                })
            })
            .unwrap(),
        );
        logger.write(Level::INFO, "first".into());
        entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        // A timeout makes a regression to blocking submissions fail, not hang.
        let producer_log = Arc::clone(&logger);
        let (done_tx, done_rx) = sync_channel(1);
        let producer = thread::spawn(move || {
            for index in 0..QUEUE_CAPACITY + 3 {
                producer_log.write(Level::INFO, format!("record {index}"));
            }
            done_tx.send(()).unwrap();
        });
        let submitted = done_rx.recv_timeout(Duration::from_secs(5));
        release_tx.send(()).unwrap();
        producer.join().unwrap();
        logger.shutdown();
        logger.shutdown();
        submitted.expect("log submission blocked on the writer");
        let records = records.lock().unwrap();
        assert_eq!(records.len(), QUEUE_CAPACITY + 2);
        assert_eq!(records[0], "first");
        assert_eq!(
            records[QUEUE_CAPACITY],
            format!("record {}", QUEUE_CAPACITY - 1)
        );
        assert!(records.last().unwrap().contains("dropped 3 log records"));
    }

    #[test]
    fn messages_are_bounded_without_splitting_utf8() {
        let (tx, rx) = sync_channel(1);
        let logger = BackgroundLog::start(move || {
            Ok(move |_, message: &str| {
                tx.send(message.to_owned()).unwrap();
            })
        })
        .unwrap();
        logger.write(Level::INFO, "€".repeat(MAX_MESSAGE_BYTES));
        let message = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(message.len(), MAX_MESSAGE_BYTES - 1);
        assert!(message.chars().all(|character| character == '€'));
        logger.shutdown();
    }

    #[test]
    fn writer_initialization_errors_are_returned() {
        let result = BackgroundLog::start(|| -> io::Result<fn(Level, &str)> {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "access denied",
            ))
        });
        assert_eq!(
            result.err().unwrap().kind(),
            io::ErrorKind::PermissionDenied
        );
    }
}
