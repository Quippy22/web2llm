use chrono::{DateTime, Utc};
use std::path::Path;

use crate::error::Result;
use crate::tokens::PageChunk;

/// The result of a successful page fetch and extraction.
pub struct PageResult {
    pub url: String,
    pub title: String,
    pub chunks: Vec<PageChunk>,
    pub timestamp: DateTime<Utc>,
}

impl PageResult {
    pub fn new(url: &str, title: &str, chunks: Vec<PageChunk>) -> Self {
        Self {
            url: url.to_string(),
            title: title.to_string(),
            chunks,
            timestamp: Utc::now(),
        }
    }

    pub fn markdown(&self) -> String {
        self.chunks
            .iter()
            .map(|c| c.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        std::fs::write(path, self.markdown())?;
        Ok(())
    }

    pub fn save_auto(&self, dir: &Path) -> Result<()> {
        std::fs::create_dir_all(dir)?;
        let filename = filename_from_url(&self.url);
        let path = dir.join(format!("{}.md", filename));
        std::fs::write(path, self.markdown())?;
        Ok(())
    }

    pub fn total_tokens(&self) -> usize {
        self.chunks.iter().map(|c| c.tokens).sum()
    }

    pub fn get_urls(&self) -> Vec<String> {
        let mut urls = Vec::new();
        let markdown = self.markdown();
        for part in markdown.split(&['(', ')', ' ', '\n', '\t', '<', '>', '[', ']', '"']) {
            if (part.starts_with("http://") || part.starts_with("https://")) && part.len() > 10 {
                urls.push(part.to_string());
            }
        }
        urls.sort();
        urls.dedup();
        urls
    }
}

fn filename_from_url(url: &str) -> String {
    url.chars()
        .map(|c| {
            if c.is_alphabetic() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_page_result_new() {
        let result = PageResult::new("https://example.com", "Example", vec![]);
        assert_eq!(result.url, "https://example.com");
        assert_eq!(result.title, "Example");
        assert_eq!(result.chunks.len(), 0);
    }

    #[test]
    fn test_page_result_save() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.md");
        let result = PageResult::new("url", "title", vec![]);
        result.save(&path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert_eq!(content, "");
    }

    #[test]
    fn test_page_result_save_auto() {
        let dir = tempdir().unwrap();
        let result = PageResult::new("https://example.com/some-page", "title", vec![]);
        result.save_auto(dir.path()).unwrap();
        let filename = filename_from_url("https://example.com/some-page");
        let path = dir.path().join(format!("{}.md", filename));
        assert!(path.exists());
        let content = std::fs::read_to_string(path).unwrap();
        assert_eq!(content, "");
    }
}
