//! Incremental compilation cache.
//!
//! Caches parsed ASTs and compiled outputs per file, keyed by content hash.
//! Only recompiles files that have changed, dramatically improving rebuild times
//! for large projects.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::ast::StyleSheet;

/// Content-addressable cache for incremental compilation.
pub struct CompilationCache {
    /// Source hash -> parsed AST
    ast_cache: HashMap<u64, CachedAST>,
    /// Source hash + config hash -> compiled CSS
    css_cache: HashMap<u64, CachedCSS>,
    /// File path -> last known source hash
    file_hashes: HashMap<String, u64>,
}

struct CachedAST {
    stylesheet: StyleSheet,
    #[allow(dead_code)]
    source_hash: u64,
}

struct CachedCSS {
    css: String,
    js: Option<String>,
    source_map: Option<String>,
}

impl CompilationCache {
    pub fn new() -> Self {
        Self {
            ast_cache: HashMap::new(),
            css_cache: HashMap::new(),
            file_hashes: HashMap::new(),
        }
    }

    /// Check if a file has changed since last compilation.
    pub fn has_changed(&self, file_path: &str, source: &str) -> bool {
        let new_hash = hash_source(source);
        match self.file_hashes.get(file_path) {
            Some(&old_hash) => old_hash != new_hash,
            None => true, // New file
        }
    }

    /// Get cached AST for a source, if available and not stale.
    pub fn get_ast(&self, source: &str) -> Option<&StyleSheet> {
        let hash = hash_source(source);
        self.ast_cache.get(&hash).map(|c| &c.stylesheet)
    }

    /// Cache an AST for a source.
    pub fn put_ast(&mut self, source: &str, stylesheet: StyleSheet) {
        let hash = hash_source(source);
        self.ast_cache.insert(hash, CachedAST {
            stylesheet,
            source_hash: hash,
        });
    }

    /// Get cached CSS output for a source + config combination.
    pub fn get_css(&self, source: &str, config_hash: u64) -> Option<(&str, Option<&str>, Option<&str>)> {
        let hash = combined_hash(hash_source(source), config_hash);
        self.css_cache.get(&hash).map(|c| {
            (c.css.as_str(), c.js.as_deref(), c.source_map.as_deref())
        })
    }

    /// Cache CSS output for a source + config combination.
    pub fn put_css(
        &mut self,
        source: &str,
        config_hash: u64,
        css: String,
        js: Option<String>,
        source_map: Option<String>,
    ) {
        let hash = combined_hash(hash_source(source), config_hash);
        self.css_cache.insert(hash, CachedCSS { css, js, source_map });
    }

    /// Update the file hash tracking.
    pub fn update_file_hash(&mut self, file_path: &str, source: &str) {
        let hash = hash_source(source);
        self.file_hashes.insert(file_path.to_string(), hash);
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            ast_entries: self.ast_cache.len(),
            css_entries: self.css_cache.len(),
            tracked_files: self.file_hashes.len(),
        }
    }

    /// Clear all caches.
    pub fn clear(&mut self) {
        self.ast_cache.clear();
        self.css_cache.clear();
        self.file_hashes.clear();
    }

    /// Evict a specific file from cache.
    pub fn evict(&mut self, file_path: &str) {
        self.file_hashes.remove(file_path);
    }
}

impl Default for CompilationCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub ast_entries: usize,
    pub css_entries: usize,
    pub tracked_files: usize,
}

/// Hash source content using a fast hasher.
fn hash_source(source: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}

/// Combine two hashes.
fn combined_hash(a: u64, b: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    a.hash(&mut hasher);
    b.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn test_cache_basic() {
        let mut cache = CompilationCache::new();

        let source = ".btn { color: red; }";
        assert!(cache.has_changed("test.wcss", source));

        cache.update_file_hash("test.wcss", source);
        assert!(!cache.has_changed("test.wcss", source));

        // Changed source
        assert!(cache.has_changed("test.wcss", ".btn { color: blue; }"));
    }

    #[test]
    fn test_cache_ast() {
        let mut cache = CompilationCache::new();

        let source = ".btn { color: red; }";
        assert!(cache.get_ast(source).is_none());

        let stylesheet = StyleSheet {
            rules: vec![],
            at_rules: vec![],
            span: Span::empty(),
        };
        cache.put_ast(source, stylesheet);

        assert!(cache.get_ast(source).is_some());
        assert!(cache.get_ast(".other { }").is_none());
    }

    #[test]
    fn test_cache_css() {
        let mut cache = CompilationCache::new();

        let source = ".btn { color: red; }";
        let config_hash = 42u64;

        assert!(cache.get_css(source, config_hash).is_none());

        cache.put_css(source, config_hash, ".btn{color:red}".to_string(), None, None);

        let cached = cache.get_css(source, config_hash);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().0, ".btn{color:red}");
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = CompilationCache::new();
        let stats = cache.stats();
        assert_eq!(stats.ast_entries, 0);

        cache.update_file_hash("a.wcss", "body {}");
        cache.update_file_hash("b.wcss", "div {}");
        assert_eq!(cache.stats().tracked_files, 2);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = CompilationCache::new();
        cache.update_file_hash("a.wcss", "body {}");
        cache.clear();
        assert!(cache.has_changed("a.wcss", "body {}"));
    }
}
