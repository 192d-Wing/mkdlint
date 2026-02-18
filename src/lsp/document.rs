//! Document management for LSP server

use crate::types::LintError;
use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use std::sync::Arc;
use std::time::Instant;
use tower_lsp::lsp_types::Url;

/// Represents a single document in the LSP server
#[derive(Debug, Clone)]
pub struct Document {
    /// Document URI
    pub uri: Url,
    /// Document content
    pub content: String,
    /// Document version (incremented on each change)
    pub version: i32,
    /// Cached lint errors from last lint
    pub cached_errors: Vec<LintError>,
    /// Last time this document was linted
    pub last_lint_time: Instant,
}

impl Document {
    /// Create a new document
    pub fn new(uri: Url, content: String, version: i32) -> Self {
        Self {
            uri,
            content,
            version,
            cached_errors: Vec::new(),
            last_lint_time: Instant::now(),
        }
    }

    /// Update the document content and version
    pub fn update(&mut self, content: String, version: i32) {
        self.content = content;
        self.version = version;
    }

    /// Update the cached lint errors
    pub fn update_errors(&mut self, errors: Vec<LintError>) {
        self.cached_errors = errors;
        self.last_lint_time = Instant::now();
    }
}

/// Manages all open documents in the LSP server
pub struct DocumentManager {
    documents: Arc<DashMap<Url, Document>>,
}

impl DocumentManager {
    /// Create a new document manager
    pub fn new() -> Self {
        Self {
            documents: Arc::new(DashMap::new()),
        }
    }

    /// Insert or update a document
    pub fn insert(&self, uri: Url, content: String, version: i32) {
        let doc = Document::new(uri.clone(), content, version);
        self.documents.insert(uri, doc);
    }

    /// Get a document by URI (returns a zero-copy Ref guard)
    pub fn get(&self, uri: &Url) -> Option<Ref<'_, Url, Document>> {
        self.documents.get(uri)
    }

    /// Update a document's content
    pub fn update(&self, uri: &Url, content: String, version: i32) {
        if let Some(mut entry) = self.documents.get_mut(uri) {
            entry.update(content, version);
        }
    }

    /// Update a document's cached errors
    pub fn update_errors(&self, uri: &Url, errors: Vec<LintError>) {
        if let Some(mut entry) = self.documents.get_mut(uri) {
            entry.update_errors(errors);
        }
    }

    /// Remove a document
    pub fn remove(&self, uri: &Url) -> Option<Document> {
        self.documents.remove(uri).map(|(_, doc)| doc)
    }

    /// Check if a document exists
    pub fn contains(&self, uri: &Url) -> bool {
        self.documents.contains_key(uri)
    }

    /// Get all document URIs
    pub fn all_uris(&self) -> Vec<Url> {
        self.documents
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
}

impl Default for DocumentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_new() {
        let uri = Url::parse("file:///tmp/test.md").unwrap();
        let content = "# Test".to_string();
        let doc = Document::new(uri.clone(), content.clone(), 1);

        assert_eq!(doc.uri, uri);
        assert_eq!(doc.content, content);
        assert_eq!(doc.version, 1);
        assert!(doc.cached_errors.is_empty());
    }

    #[test]
    fn test_document_update() {
        let uri = Url::parse("file:///tmp/test.md").unwrap();
        let mut doc = Document::new(uri, "# Test".to_string(), 1);

        doc.update("# Updated".to_string(), 2);
        assert_eq!(doc.content, "# Updated");
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_document_manager_insert_and_get() {
        let manager = DocumentManager::new();
        let uri = Url::parse("file:///tmp/test.md").unwrap();

        manager.insert(uri.clone(), "# Test".to_string(), 1);

        let doc = manager.get(&uri);
        assert!(doc.is_some());
        assert_eq!(doc.unwrap().content, "# Test");
    }

    #[test]
    fn test_document_manager_update() {
        let manager = DocumentManager::new();
        let uri = Url::parse("file:///tmp/test.md").unwrap();

        manager.insert(uri.clone(), "# Test".to_string(), 1);
        manager.update(&uri, "# Updated".to_string(), 2);

        let doc = manager.get(&uri).unwrap();
        assert_eq!(doc.content, "# Updated");
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_document_manager_remove() {
        let manager = DocumentManager::new();
        let uri = Url::parse("file:///tmp/test.md").unwrap();

        manager.insert(uri.clone(), "# Test".to_string(), 1);
        assert!(manager.contains(&uri));

        let removed = manager.remove(&uri);
        assert!(removed.is_some());
        assert!(!manager.contains(&uri));
    }

    #[test]
    fn test_document_manager_all_uris() {
        let manager = DocumentManager::new();
        let uri1 = Url::parse("file:///tmp/test1.md").unwrap();
        let uri2 = Url::parse("file:///tmp/test2.md").unwrap();

        manager.insert(uri1.clone(), "# Test 1".to_string(), 1);
        manager.insert(uri2.clone(), "# Test 2".to_string(), 1);

        let uris = manager.all_uris();
        assert_eq!(uris.len(), 2);
        assert!(uris.contains(&uri1));
        assert!(uris.contains(&uri2));
    }

    #[test]
    fn test_document_manager_get_returns_ref() {
        let manager = DocumentManager::new();
        let uri = Url::parse("file:///tmp/test.md").unwrap();
        manager.insert(uri.clone(), "# Test".to_string(), 1);

        // Verify Ref guard provides read access via Deref
        {
            let doc_ref = manager.get(&uri).unwrap();
            assert_eq!(doc_ref.content, "# Test");
            assert_eq!(doc_ref.version, 1);
            assert!(doc_ref.cached_errors.is_empty());
        }

        // After dropping the Ref, mutation is unblocked
        manager.update(&uri, "# Updated".to_string(), 2);
        let doc_ref = manager.get(&uri).unwrap();
        assert_eq!(doc_ref.content, "# Updated");
        assert_eq!(doc_ref.version, 2);
    }
}
