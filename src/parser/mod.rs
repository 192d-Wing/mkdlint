//! Markdown parsing functionality

mod token;

pub use token::*;

use std::collections::HashMap;

use comrak::{
    Arena, Options,
    nodes::{AstNode, NodeValue},
};

/// Parse markdown content into tokens
pub fn parse(content: &str) -> Vec<Token> {
    let arena = Arena::new();
    let mut options = Options::default();

    // Enable GFM extensions
    options.extension.strikethrough = true;
    options.extension.tagfilter = false;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.description_lists = true;
    options.extension.math_dollars = true;

    let root = comrak::parse_document(&arena, content, &options);

    let mut tokens = Vec::new();
    collect_tokens(root, &mut tokens, None);

    tokens
}

/// Recursively collect text content from a node's children
fn collect_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut text = String::new();
    let data = node.data.borrow();

    match &data.value {
        NodeValue::Text(cow) => text.push_str(cow),
        NodeValue::Code(code) => text.push_str(&code.literal),
        NodeValue::SoftBreak | NodeValue::LineBreak => text.push('\n'),
        _ => {}
    }

    for child in node.children() {
        text.push_str(&collect_text(child));
    }

    text
}

/// Recursively collect tokens from AST with parent tracking
fn collect_tokens<'a>(node: &'a AstNode<'a>, tokens: &mut Vec<Token>, parent_idx: Option<usize>) {
    let data = node.data.borrow();

    if let Some(mut token) = node_to_token(&data.value, &data.sourcepos) {
        // Collect text from child nodes for non-leaf nodes
        match &data.value {
            NodeValue::Text(_)
            | NodeValue::Code(_)
            | NodeValue::HtmlInline(_)
            | NodeValue::HtmlBlock(_) => {
                // Text already set in node_to_token
            }
            NodeValue::CodeBlock(_) => {
                // literal already set in node_to_token
            }
            _ => {
                // Collect text from children
                let text = collect_text(node);
                if !text.is_empty() {
                    token.text = text;
                }
            }
        }

        token.parent = parent_idx;
        let my_idx = tokens.len();

        // Register this token as a child of its parent
        if let Some(pidx) = parent_idx {
            tokens[pidx].children.push(my_idx);
        }

        tokens.push(token);

        // Drop the borrow before recursing
        drop(data);

        // Process children
        for child in node.children() {
            collect_tokens(child, tokens, Some(my_idx));
        }
    } else {
        // Skip this node but still process children (e.g. for Document)
        drop(data);
        for child in node.children() {
            collect_tokens(child, tokens, parent_idx);
        }
    }
}

/// Convert comrak NodeValue to Token with metadata
fn node_to_token(value: &NodeValue, sourcepos: &comrak::nodes::Sourcepos) -> Option<Token> {
    let token_type;
    let mut text = String::new();
    let mut metadata = HashMap::new();

    match value {
        NodeValue::Document => return None,
        NodeValue::BlockQuote => token_type = "blockQuote",
        NodeValue::List(nl) => {
            token_type = "list";
            let ordered = matches!(nl.list_type, comrak::nodes::ListType::Ordered);
            metadata.insert("ordered".to_string(), ordered.to_string());
            metadata.insert("start".to_string(), nl.start.to_string());
            metadata.insert("tight".to_string(), nl.tight.to_string());
            metadata.insert(
                "bullet_char".to_string(),
                (nl.bullet_char as char).to_string(),
            );
            let delim = match nl.delimiter {
                comrak::nodes::ListDelimType::Period => ".",
                comrak::nodes::ListDelimType::Paren => ")",
            };
            metadata.insert("delimiter".to_string(), delim.to_string());
        }
        NodeValue::Item(_) => token_type = "listItem",
        NodeValue::DescriptionList => token_type = "descriptionList",
        NodeValue::DescriptionItem(_) => token_type = "descriptionItem",
        NodeValue::DescriptionTerm => token_type = "descriptionTerm",
        NodeValue::DescriptionDetails => token_type = "descriptionDetails",
        NodeValue::CodeBlock(cb) => {
            token_type = "codeBlock";
            text = cb.literal.clone();
            metadata.insert("info".to_string(), cb.info.clone());
            metadata.insert("fenced".to_string(), cb.fenced.to_string());
            metadata.insert(
                "fence_char".to_string(),
                (cb.fence_char as char).to_string(),
            );
            metadata.insert("fence_length".to_string(), cb.fence_length.to_string());
        }
        NodeValue::HtmlBlock(hb) => {
            token_type = "htmlBlock";
            text = hb.literal.clone();
        }
        NodeValue::Paragraph => token_type = "paragraph",
        NodeValue::Heading(h) => {
            token_type = "heading";
            metadata.insert("level".to_string(), h.level.to_string());
            metadata.insert("setext".to_string(), h.setext.to_string());
        }
        NodeValue::ThematicBreak => token_type = "thematicBreak",
        NodeValue::FootnoteDefinition(fndef) => {
            token_type = "footnoteDefinition";
            metadata.insert("name".to_string(), fndef.name.clone());
        }
        NodeValue::Table(table) => {
            token_type = "table";
            metadata.insert("columns".to_string(), table.num_columns.to_string());
        }
        NodeValue::TableRow(header) => {
            token_type = "tableRow";
            metadata.insert("header".to_string(), header.to_string());
        }
        NodeValue::TableCell => token_type = "tableCell",
        NodeValue::Text(cow) => {
            token_type = "text";
            text = cow.to_string();
        }
        NodeValue::TaskItem(task) => {
            token_type = "taskItem";
            let is_checked = task.symbol.is_some();
            metadata.insert("checked".to_string(), is_checked.to_string());
        }
        NodeValue::SoftBreak => token_type = "softBreak",
        NodeValue::LineBreak => token_type = "lineBreak",
        NodeValue::Code(code) => {
            token_type = "code";
            text = code.literal.clone();
        }
        NodeValue::HtmlInline(html) => {
            token_type = "htmlInline";
            text = html.to_string();
        }
        NodeValue::Emph => token_type = "emphasis",
        NodeValue::Strong => token_type = "strong",
        NodeValue::Strikethrough => token_type = "strikethrough",
        NodeValue::Superscript => token_type = "superscript",
        NodeValue::Link(link) => {
            token_type = "link";
            metadata.insert("url".to_string(), link.url.clone());
            metadata.insert("title".to_string(), link.title.clone());
        }
        NodeValue::Image(img) => {
            token_type = "image";
            metadata.insert("url".to_string(), img.url.clone());
            metadata.insert("title".to_string(), img.title.clone());
        }
        NodeValue::FootnoteReference(fnref) => {
            token_type = "footnoteReference";
            metadata.insert("name".to_string(), fnref.name.clone());
        }
        NodeValue::Math(math) => {
            token_type = "math";
            text = math.literal.clone();
            metadata.insert("display".to_string(), math.display_math.to_string());
        }
        _ => return None,
    }

    Some(Token {
        token_type: token_type.to_string(),
        start_line: sourcepos.start.line,
        start_column: sourcepos.start.column,
        end_line: sourcepos.end.line,
        end_column: sourcepos.end.column,
        text,
        children: Vec::new(),
        parent: None,
        metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let markdown = "# Hello\n\nThis is a paragraph.";
        let tokens = parse(markdown);

        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_parse_heading() {
        let markdown = "# Heading 1\n## Heading 2";
        let tokens = parse(markdown);

        let headings: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == "heading")
            .collect();

        assert_eq!(headings.len(), 2);
    }

    #[test]
    fn test_heading_metadata() {
        let markdown = "# Heading 1\n## Heading 2\n### Heading 3";
        let tokens = parse(markdown);

        let headings: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == "heading")
            .collect();

        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0].metadata.get("level").unwrap(), "1");
        assert_eq!(headings[1].metadata.get("level").unwrap(), "2");
        assert_eq!(headings[2].metadata.get("level").unwrap(), "3");
        assert_eq!(headings[0].metadata.get("setext").unwrap(), "false");
    }

    #[test]
    fn test_heading_text_content() {
        let markdown = "# Hello World";
        let tokens = parse(markdown);

        let headings: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == "heading")
            .collect();

        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].text, "Hello World");
    }

    #[test]
    fn test_setext_heading() {
        let markdown = "Heading 1\n=========\n\nHeading 2\n---------";
        let tokens = parse(markdown);

        let headings: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == "heading")
            .collect();

        assert_eq!(headings.len(), 2);
        assert_eq!(headings[0].metadata.get("level").unwrap(), "1");
        assert_eq!(headings[0].metadata.get("setext").unwrap(), "true");
        assert_eq!(headings[1].metadata.get("level").unwrap(), "2");
        assert_eq!(headings[1].metadata.get("setext").unwrap(), "true");
    }

    #[test]
    fn test_code_block_metadata() {
        let markdown = "```rust\nfn main() {}\n```";
        let tokens = parse(markdown);

        let code_blocks: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == "codeBlock")
            .collect();

        assert_eq!(code_blocks.len(), 1);
        assert_eq!(code_blocks[0].metadata.get("info").unwrap(), "rust");
        assert!(code_blocks[0].text.contains("fn main()"));
    }

    #[test]
    fn test_link_metadata() {
        let markdown = "[click here](https://example.com \"Example\")";
        let tokens = parse(markdown);

        let links: Vec<_> = tokens.iter().filter(|t| t.token_type == "link").collect();

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].metadata.get("url").unwrap(), "https://example.com");
        assert_eq!(links[0].metadata.get("title").unwrap(), "Example");
        assert_eq!(links[0].text, "click here");
    }

    #[test]
    fn test_parent_child_relationships() {
        let markdown = "# Hello\n\nA paragraph.";
        let tokens = parse(markdown);

        // Find the heading token
        let heading_idx = tokens
            .iter()
            .position(|t| t.token_type == "heading")
            .unwrap();

        let heading = &tokens[heading_idx];
        // Heading should have children (at least a text node)
        assert!(!heading.children.is_empty());

        // Children should reference the heading as parent
        for &child_idx in &heading.children {
            assert_eq!(tokens[child_idx].parent, Some(heading_idx));
        }
    }

    #[test]
    fn test_list_metadata() {
        let markdown = "1. First\n2. Second\n3. Third";
        let tokens = parse(markdown);

        let lists: Vec<_> = tokens.iter().filter(|t| t.token_type == "list").collect();

        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].metadata.get("ordered").unwrap(), "true");
        assert_eq!(lists[0].metadata.get("start").unwrap(), "1");
    }
}
