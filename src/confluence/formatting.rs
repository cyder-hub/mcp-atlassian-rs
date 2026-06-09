use serde_json::Value;

use crate::atlassian::error::AtlassianError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfluenceContentFormat {
    Markdown,
    Storage,
    Html,
    Wiki,
}

impl ConfluenceContentFormat {
    pub fn parse(value: Option<&str>) -> Result<Self, AtlassianError> {
        match value
            .unwrap_or("markdown")
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "" | "markdown" => Ok(Self::Markdown),
            "storage" => Ok(Self::Storage),
            "html" => Ok(Self::Html),
            "wiki" => Ok(Self::Wiki),
            _ => Err(AtlassianError::invalid_input(
                "content_format must be markdown, storage, html, or wiki",
            )),
        }
    }
}

pub fn content_to_storage(value: &str, format: ConfluenceContentFormat) -> String {
    match format {
        ConfluenceContentFormat::Storage | ConfluenceContentFormat::Html => value.to_string(),
        ConfluenceContentFormat::Markdown | ConfluenceContentFormat::Wiki => {
            markdown_to_storage(value)
        }
    }
}

pub fn markdown_to_storage(markdown: &str) -> String {
    let mut blocks = Vec::new();

    for block in markdown.split("\n\n") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        if let Some(code) = fenced_code_block(block) {
            blocks.push(format!(
                "<pre><code>{}</code></pre>",
                escape_storage_text(code)
            ));
            continue;
        }

        if is_unordered_list_block(block) {
            let items = block
                .lines()
                .filter_map(unordered_list_item_text)
                .map(|item| format!("<li>{}</li>", render_inline_storage(item)))
                .collect::<Vec<_>>()
                .join("");
            blocks.push(format!("<ul>{items}</ul>"));
            continue;
        }

        let (tag, text) = if let Some(text) = block.strip_prefix("### ") {
            ("h3", text)
        } else if let Some(text) = block.strip_prefix("## ") {
            ("h2", text)
        } else if let Some(text) = block.strip_prefix("# ") {
            ("h1", text)
        } else {
            ("p", block)
        };

        let rendered = render_inline_storage(text).replace('\n', "<br />");
        blocks.push(format!("<{tag}>{rendered}</{tag}>"));
    }

    if blocks.is_empty() {
        "<p></p>".to_string()
    } else {
        blocks.join("")
    }
}

pub fn storage_to_markdown(value: &str) -> String {
    strip_html_tags(value)
}

pub fn body_value_as_markdown(body: &Value) -> Option<String> {
    body.pointer("/storage/value")
        .or_else(|| body.pointer("/view/value"))
        .or_else(|| body.pointer("/export_view/value"))
        .and_then(Value::as_str)
        .map(storage_to_markdown)
}

pub fn body_value_as_storage(body: &Value) -> Option<String> {
    body.pointer("/storage/value")
        .or_else(|| body.pointer("/view/value"))
        .or_else(|| body.pointer("/export_view/value"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

pub fn strip_html_tags(value: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    let mut last_was_space = false;

    for character in value.chars() {
        match character {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                push_space(&mut output, &mut last_was_space);
            }
            _ if in_tag => {}
            '&' => {
                output.push('&');
                last_was_space = false;
            }
            character if character.is_whitespace() => push_space(&mut output, &mut last_was_space),
            character => {
                output.push(character);
                last_was_space = false;
            }
        }
    }

    decode_minimal_entities(output.trim()).to_string()
}

pub fn escape_storage_text(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            _ => output.push(character),
        }
    }
    output
}

fn fenced_code_block(block: &str) -> Option<&str> {
    let after_opening = block.strip_prefix("```")?;
    let content_start = after_opening.find('\n')? + 4;
    let content_end = block.rfind("\n```")?;
    if content_end < content_start {
        return None;
    }
    Some(&block[content_start..content_end])
}

fn is_unordered_list_block(block: &str) -> bool {
    block
        .lines()
        .all(|line| unordered_list_item_text(line).is_some())
}

fn unordered_list_item_text(line: &str) -> Option<&str> {
    line.trim()
        .strip_prefix("- ")
        .or_else(|| line.trim().strip_prefix("* "))
}

fn render_inline_storage(value: &str) -> String {
    let mut output = String::new();
    let mut rest = value;

    while let Some(open_index) = rest.find('[') {
        let after_open = &rest[open_index + 1..];
        let Some(label_end) = after_open.find("](") else {
            break;
        };
        let href_start = open_index + 1 + label_end + 2;
        let Some(href_end_relative) = rest[href_start..].find(')') else {
            break;
        };
        let href_end = href_start + href_end_relative;
        let label = &after_open[..label_end];
        let href = &rest[href_start..href_end];

        output.push_str(&escape_storage_text(&rest[..open_index]));
        output.push_str("<a href=\"");
        output.push_str(&escape_storage_text(href));
        output.push_str("\">");
        output.push_str(&escape_storage_text(label));
        output.push_str("</a>");
        rest = &rest[href_end + 1..];
    }

    output.push_str(&escape_storage_text(rest));
    output
}

pub fn safe_path_segment(segment: &str, name: &'static str) -> Result<String, AtlassianError> {
    let segment = segment.trim();
    if segment.is_empty() || segment.contains('/') || segment.contains('?') || segment.contains('#')
    {
        Err(AtlassianError::invalid_input(format!(
            "{name} must be a non-empty path segment"
        )))
    } else {
        Ok(segment.to_string())
    }
}

fn push_space(output: &mut String, last_was_space: &mut bool) {
    if !*last_was_space && !output.is_empty() {
        output.push(' ');
        *last_was_space = true;
    }
}

fn decode_minimal_entities(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn markdown_to_storage_is_deterministic_and_escapes_text() {
        assert_eq!(
            markdown_to_storage("# Title\n\nHello <team> & \"docs\""),
            "<h1>Title</h1><p>Hello &lt;team&gt; &amp; &quot;docs&quot;</p>"
        );
    }

    #[test]
    fn markdown_to_storage_snapshot_covers_current_boundary() {
        let markdown = "# Roadmap\n\nHello <team> & \"docs\"\n\n- first item\n- [second](https://example.invalid?q=1&x=<tag>)\n\n```text\n<raw & value>\n```";

        assert_eq!(
            markdown_to_storage(markdown),
            concat!(
                "<h1>Roadmap</h1>",
                "<p>Hello &lt;team&gt; &amp; &quot;docs&quot;</p>",
                "<ul><li>first item</li>",
                "<li><a href=\"https://example.invalid?q=1&amp;x=&lt;tag&gt;\">second</a></li></ul>",
                "<pre><code>&lt;raw &amp; value&gt;</code></pre>"
            )
        );
    }

    #[test]
    fn storage_to_markdown_strips_html_and_decodes_minimal_entities() {
        assert_eq!(
            storage_to_markdown("<h1>Title</h1><p>Hello &amp; welcome</p>"),
            "Title Hello & welcome"
        );
    }

    #[test]
    fn body_value_helpers_prefer_storage_then_view() {
        let body = json!({
            "storage": {"value": "<p>Storage</p>"},
            "view": {"value": "<p>View</p>"}
        });

        assert_eq!(body_value_as_markdown(&body).as_deref(), Some("Storage"));
        assert_eq!(
            body_value_as_storage(&body).as_deref(),
            Some("<p>Storage</p>")
        );
    }

    #[test]
    fn content_format_parser_rejects_unknown_values() {
        let error = ConfluenceContentFormat::parse(Some("adf")).unwrap_err();

        assert!(error.to_string().contains("content_format"));
    }
}
