use actix_web::web::Bytes;
use std::io::Write;
use std::mem;
use std::pin::Pin;
use tokio::sync::mpsc;

/// The response writer is a buffered async writer that sends data to the client.
/// Writing to it just appends to an in-memory buffer, which is flushed to the client asynchronously
/// when `async_flush()` is called.
/// This allows streaming data to the client without blocking, and has built-in back-pressure:
/// if the client cannot keep up with the data, `async_flush()` will fill the sending queue,
/// then block until the client has consumed some data.
#[derive(Clone)]
pub struct ResponseWriter {
    buffer: Vec<u8>,
    response_bytes: mpsc::Sender<Bytes>,
}

impl ResponseWriter {
    #[must_use]
    pub fn new(response_bytes: mpsc::Sender<Bytes>) -> Self {
        Self {
            response_bytes,
            buffer: Vec::new(),
        }
    }

    pub async fn close_with_error(&mut self, mut msg: String) {
        if !self.response_bytes.is_closed() {
            if let Err(e) = self.async_flush().await {
                use std::fmt::Write;
                write!(&mut msg, "Unable to flush data: {e}").unwrap();
            }
            if let Err(e) = self.response_bytes.send(msg.into()).await {
                log::error!("Unable to send error back to client: {e}");
            }
        }
    }

    pub async fn async_flush(&mut self) -> std::io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        log::trace!(
            "Flushing data to client: {}",
            String::from_utf8_lossy(&self.buffer)
        );
        let sender = self
            .response_bytes
            .reserve()
            .await
            .map_err(|_| std::io::ErrorKind::WouldBlock)?;
        sender.send(std::mem::take(&mut self.buffer).into());
        Ok(())
    }
}

impl Write for ResponseWriter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        log::trace!(
            "Flushing data to client: {}",
            String::from_utf8_lossy(&self.buffer)
        );
        self.response_bytes
            .try_send(mem::take(&mut self.buffer).into())
            .map_err(|e|
                std::io::Error::new(
                    std::io::ErrorKind::WouldBlock,
                    format!("{e}: Row limit exceeded. The server cannot store more than {} pending messages in memory. Try again later or increase max_pending_rows in the configuration.", self.response_bytes.max_capacity())
                )
            )
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct AsyncResponseWriter {
    poll_sender: tokio_util::sync::PollSender<Bytes>,
    writer: ResponseWriter,
}

impl AsyncResponseWriter {
    #[must_use]
    pub fn new(writer: ResponseWriter) -> Self {
        let sender = writer.response_bytes.clone();
        Self {
            poll_sender: tokio_util::sync::PollSender::new(sender),
            writer,
        }
    }

    #[must_use]
    pub fn into_inner(self) -> ResponseWriter {
        self.writer
    }
}

impl tokio::io::AsyncWrite for AsyncResponseWriter {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::task::Poll::Ready(self.as_mut().writer.write(buf))
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let Self {
            poll_sender,
            writer,
        } = self.get_mut();
        match poll_sender.poll_reserve(cx) {
            std::task::Poll::Ready(Ok(())) => {
                let res = poll_sender.send_item(std::mem::take(&mut writer.buffer).into());
                std::task::Poll::Ready(res.map_err(|_| std::io::ErrorKind::BrokenPipe.into()))
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
            std::task::Poll::Ready(Err(_e)) => {
                std::task::Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()))
            }
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        self.poll_flush(cx)
    }
}

impl Drop for ResponseWriter {
    fn drop(&mut self) {
        if let Err(e) = std::io::Write::flush(self) {
            log::error!("Could not flush data to client: {e}");
        }
    }
}
