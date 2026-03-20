/// Performs post-processing on the generated Markdown content to reduce
/// token count and improve readability.
///
/// Currently handles:
/// 1. Collapsing multiple empty lines into a single newline.
/// 2. Trimming leading/trailing whitespace.
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
