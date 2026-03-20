//! Post-processing utilities for Markdown content.
//!
//! This module provides functions to "wash" and optimize Markdown after it
//! has been converted from HTML, ensuring it is as token-efficient as possible.

/// Performs post-processing on the generated Markdown content to reduce
/// token count and improve readability.
///
/// Optimization steps:
/// 1. Collapsing multiple empty lines into a single newline to remove excessive vertical whitespace.
/// 2. Trimming leading and trailing whitespace from each line.
/// 3. Removing trailing empty lines from the entire document.
pub fn wash_markdown(content: &str) -> String {
    let mut washed = String::with_capacity(content.len());
    let mut last_was_newline = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !last_was_newline {
                washed.push('\n');
                last_was_newline = true;
            }
        } else {
            washed.push_str(trimmed);
            washed.push('\n');
            last_was_newline = false;
        }
    }

    washed.trim().to_string()
}
