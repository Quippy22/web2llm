use chrono::{DateTime, Utc};

pub struct PageResult {
    pub url: String,
    pub title: String,
    pub markdown: String,
    pub timestamp: DateTime<Utc>,
}

impl PageResult {
    pub fn new(url: &str, title: &str, markdown: String) -> Self {
        Self {
            url: url.to_string(),
            title: title.to_string(),
            markdown,
            timestamp: Utc::now(),
        }
    }
}
