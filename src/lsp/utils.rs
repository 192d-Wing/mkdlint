//! Utility functions for LSP implementation

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::AbortHandle;
use tower_lsp::lsp_types::{Position, Range, Url};

/// Convert a file:// URI to a PathBuf
pub fn uri_to_path(uri: &Url) -> Option<PathBuf> {
    uri.to_file_path().ok()
}

/// Convert a PathBuf to a file:// URI
pub fn path_to_uri(path: &PathBuf) -> Option<Url> {
    Url::from_file_path(path).ok()
}

/// Convert 1-based line/column to LSP Position (0-based)
pub fn to_position(line: usize, column: usize) -> Position {
    Position {
        line: (line.saturating_sub(1)) as u32,
        character: (column.saturating_sub(1)) as u32,
    }
}

/// Convert (line, col, len) to LSP Range
pub fn to_range(line: usize, column: usize, length: usize) -> Range {
    let start = to_position(line, column);
    let end = Position {
        line: start.line,
        character: start.character + length as u32,
    };
    Range { start, end }
}

/// Debouncer for delaying operations until user stops typing
pub struct Debouncer {
    pending_tasks: Arc<dashmap::DashMap<Url, AbortHandle>>,
    delay: Duration,
}

impl Debouncer {
    /// Create a new debouncer with the given delay
    pub fn new(delay: Duration) -> Self {
        Self {
            pending_tasks: Arc::new(dashmap::DashMap::new()),
            delay,
        }
    }

    /// Schedule a task to run after the delay
    /// Cancels any previously scheduled task for the same URI
    pub fn schedule<F>(&self, uri: Url, task: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        // Cancel existing task if any
        if let Some((_, handle)) = self.pending_tasks.remove(&uri) {
            handle.abort();
        }

        // Spawn new task with delay
        let delay = self.delay;
        let pending_tasks = Arc::clone(&self.pending_tasks);
        let uri_clone = uri.clone();

        let handle = tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            task.await;
            pending_tasks.remove(&uri_clone);
        })
        .abort_handle();

        self.pending_tasks.insert(uri, handle);
    }

    /// Cancel any pending task for the given URI
    pub fn cancel(&self, uri: &Url) {
        if let Some((_, handle)) = self.pending_tasks.remove(uri) {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::Mutex;

    #[test]
    fn test_to_position() {
        assert_eq!(to_position(1, 1), Position::new(0, 0));
        assert_eq!(to_position(5, 10), Position::new(4, 9));
        assert_eq!(to_position(0, 0), Position::new(0, 0)); // Edge case
    }

    #[test]
    fn test_to_range() {
        let range = to_range(1, 1, 5);
        assert_eq!(range.start, Position::new(0, 0));
        assert_eq!(range.end, Position::new(0, 5));
    }

    #[test]
    fn test_uri_to_path() {
        let uri = Url::parse("file:///tmp/test.md").unwrap();
        let path = uri_to_path(&uri);
        assert!(path.is_some());
        assert_eq!(path.unwrap(), PathBuf::from("/tmp/test.md"));
    }

    #[test]
    #[cfg(unix)]
    fn test_path_to_uri() {
        let path = PathBuf::from("/tmp/test.md");
        let uri = path_to_uri(&path);
        assert!(uri.is_some());
        assert_eq!(uri.unwrap().scheme(), "file");
    }

    #[tokio::test]
    async fn test_debouncer() {
        let debouncer = Debouncer::new(Duration::from_millis(50));
        let uri = Url::parse("file:///tmp/test.md").unwrap();

        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);

        // Schedule a task
        debouncer.schedule(uri.clone(), async move {
            let mut count = counter_clone.lock().await;
            *count += 1;
        });

        // Wait for task to complete — use a generous timeout for slow CI runners
        tokio::time::sleep(Duration::from_millis(500)).await;

        assert_eq!(*counter.lock().await, 1);
    }

    #[tokio::test]
    async fn test_debouncer_cancel() {
        let debouncer = Debouncer::new(Duration::from_millis(50));
        let uri = Url::parse("file:///tmp/test.md").unwrap();

        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);

        // Schedule a task
        debouncer.schedule(uri.clone(), async move {
            let mut count = counter_clone.lock().await;
            *count += 1;
        });

        // Cancel it immediately
        debouncer.cancel(&uri);

        // Wait to ensure task doesn't run
        tokio::time::sleep(Duration::from_millis(500)).await;

        assert_eq!(*counter.lock().await, 0);
    }
}
