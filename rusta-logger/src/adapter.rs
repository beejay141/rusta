use std::cell::RefCell;

use crate::types::LogEntry;

/// Formats a [`LogEntry`] into a string (no trailing newline).
pub trait LogAdapter: Send + Sync {
    fn format(&self, entry: &LogEntry) -> String;
}

/// Default adapter: serializes the entry as a compact JSON line.
///
/// Uses a thread-local scratch buffer to avoid allocating a fresh `String`
/// on every call. The buffer grows to the largest entry seen on that thread
/// and is then reused. To avoid an extra copy we swap the underlying
/// `Vec<u8>` out of the TLS slot and convert it directly into a `String`.
pub struct DefaultJsonAdapter;

thread_local! {
    /// Reusable buffer for JSON serialisation. Grows to the high-water mark
    /// and then stabilises — no allocations after the first few entries.
    static JSON_BUF: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(2048));
}

impl LogAdapter for DefaultJsonAdapter {
    fn format(&self, entry: &LogEntry) -> String {
        JSON_BUF.with(|cell| {
            let mut buf = cell.borrow_mut();
            buf.clear();

            if let Err(e) = serde_json::to_writer(&mut *buf, entry) {
                // On serialisation failure write an inline error object.
                let err = format!(r#"{{"error":"failed to serialize log entry: {}"}}"#, e);
                buf.extend_from_slice(err.as_bytes());
            }

            // Swap the buffer out so we can return it without copying.
            let cap = buf.capacity();
            let owned = std::mem::replace(&mut *buf, Vec::with_capacity(cap));
            // SAFETY: serde_json and the error path above produce valid UTF-8.
            unsafe { String::from_utf8_unchecked(owned) }
        })
    }
}