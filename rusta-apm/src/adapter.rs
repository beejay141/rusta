use crate::types::ApmEntry;
use std::cell::RefCell;

/// Formats an [`ApmEntry`] into an NDJSON line string (no trailing newline).
pub trait LogAdapter: Send + Sync {
    fn format(&self, entry: &ApmEntry) -> String;
}

/// Default adapter: serializes the entry as a compact JSON line using a
/// thread-local buffer to avoid frequent allocations.
pub struct DefaultJsonAdapter;

thread_local! {
    static JSON_BUF: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(2048));
}

impl LogAdapter for DefaultJsonAdapter {
    fn format(&self, entry: &ApmEntry) -> String {
        JSON_BUF.with(|cell| {
            let mut buf = cell.borrow_mut();
            buf.clear();

            if let Err(e) = serde_json::to_writer(&mut *buf, entry) {
                let err = format!(r#"{{"error":"failed to serialize APM entry: {}"}}"#, e);
                buf.extend_from_slice(err.as_bytes());
            }

            let cap = buf.capacity();
            let owned = std::mem::replace(&mut *buf, Vec::with_capacity(cap));
            unsafe { String::from_utf8_unchecked(owned) }
        })
    }
}
