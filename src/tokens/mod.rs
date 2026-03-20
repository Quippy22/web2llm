use scraper::ElementRef;

mod optimize;
pub use optimize::wash_markdown;

/// A structurally-aware slice of the page's content converted to Markdown.
#[derive(Debug, Clone)]
pub struct PageChunk {
    /// The index of this chunk in the original document (0-indexed).
    pub index: usize,
    /// The cleaned Markdown content of the chunk.
    pub content: String,
    /// The estimated number of tokens in the Markdown content.
    pub tokens: usize,
    /// The combined score of all nodes that make up this chunk.
    pub score: f32,
}

/// Estimates the number of tokens and words in the direct text children of `node`.
pub(crate) fn get_direct_text_metrics(node: ElementRef<'_>) -> (f32, usize) {
    let mut total_words = 0.0;
    let mut total_tokens = 0;
    let mut char_in_word = 0;

    for child in node.children() {
        if let Some(text) = child.value().as_text() {
            let mut in_word = false;
            for c in text.chars() {
                if c.is_whitespace() {
                    if in_word && char_in_word > 0 {
                        total_tokens += 1;
                        char_in_word = 0;
                    }
                    in_word = false;
                } else {
                    if !in_word {
                        total_words += 1.0;
                        in_word = true;
                    }
                    char_in_word += 1;
                    if char_in_word == 4 {
                        total_tokens += 1;
                        char_in_word = 0;
                    }
                }
            }
            if in_word && char_in_word > 0 {
                total_tokens += 1;
                char_in_word = 0;
            }
        }
    }
    (total_words, total_tokens)
}

/// Checks if a token count is within the budget, allowing a 10% "soft limit".
pub(crate) fn is_within_budget(tokens: usize, max: usize) -> bool {
    tokens <= (max as f64 * 1.1) as usize
}
