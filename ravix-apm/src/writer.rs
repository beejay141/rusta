use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

// ── Split design ───────────────────────────────────────────────────────────
// ApmWriter  = cheaply cloneable channel sender (hot path, lock-free)
// ApmWriterHandle = sender + join handle (kept in shutdown registry)
// ───────────────────────────────────────────────────────────────────────────

/// Cheaply-cloneable handle used on the hot path to enqueue APM lines.
///
/// Internally wraps an [`UnboundedSender`] (which is just an `Arc`), so
/// `Clone` is essentially free.
#[derive(Clone)]
pub struct ApmWriter {
    sender: UnboundedSender<String>,
}

impl ApmWriter {
    /// Enqueue a formatted line for writing (non-blocking).
    pub fn write_line(&self, line: String) {
        let _ = self.sender.send(line);
    }
}

/// Owned handle returned by [`ApmWriterHandle::new`].
///
/// Dropping this handle (or calling [`shutdown`](Self::shutdown)) drops the
/// last sender, which signals the background writer task to drain and exit.
pub struct ApmWriterHandle {
    sender: UnboundedSender<String>,
    join_handle: tokio::task::JoinHandle<()>,
}

impl ApmWriterHandle {
    /// Open (or create) the log file in append mode and spawn the writer task.
    pub async fn new(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;

        let (sender, receiver) = mpsc::unbounded_channel();
        let join_handle = tokio::spawn(writer_loop(file, receiver));

        Ok(Self {
            sender,
            join_handle,
        })
    }

    /// Return a cheaply-cloneable [`ApmWriter`] for the hot path.
    pub fn writer(&self) -> ApmWriter {
        ApmWriter {
            sender: self.sender.clone(),
        }
    }

    /// Drop the sender and wait for all queued entries to flush.
    pub async fn shutdown(self) {
        drop(self.sender);
        let _ = self.join_handle.await;
    }
}

/// Maximum number of lines to buffer before flushing.
/// Under concurrent load this limits how many writes get batched.
const BATCH_SIZE: usize = 64;

async fn writer_loop(mut file: fs::File, mut receiver: UnboundedReceiver<String>) {
    // Pre-allocated buffer to join line + newline into a single write.
    let mut buf = String::with_capacity(4096);
    let mut count: usize = 0;

    while let Some(line) = receiver.recv().await {
        // Build "line\n" into our reusable buffer.
        buf.clear();
        buf.push_str(&line);
        buf.push('\n');

        if let Err(e) = file.write_all(buf.as_bytes()).await {
            log::warn!("ravix-apm writer: failed to write entry: {}", e);
            continue;
        }

        count += 1;

        // Batch flush: only fsync every N lines to reduce syscall pressure.
        if count >= BATCH_SIZE {
            if let Err(e) = file.flush().await {
                log::warn!("ravix-apm writer: failed to flush: {}", e);
            }
            count = 0;
        }
    }
}
