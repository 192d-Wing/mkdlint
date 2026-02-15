//! Markdown parsing functionality

mod token;

pub use token::*;

use comrak::{
    nodes::{AstNode, NodeValue},
    Arena, Options,
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

    let root = comrak::parse_document(&arena, content, &options);

    let mut tokens = Vec::new();
    collect_tokens(root, content, &mut tokens);

    tokens
}

/// Recursively collect tokens from AST
fn collect_tokens<'a>(node: &'a AstNode<'a>, content: &str, tokens: &mut Vec<Token>) {
    let data = node.data.borrow();

    // Convert comrak node to our Token type
    if let Some(token) = node_to_token(&data.value, &data.sourcepos, content) {
        let _token_index = tokens.len();
        tokens.push(token);

        // Process children
        for child in node.children() {
            collect_tokens(child, content, tokens);
        }

        // Set children indices (would need to track this differently in real implementation)
        // This is a simplified version
    }
}

/// Convert comrak NodeValue to Token
fn node_to_token(
    value: &NodeValue,
    sourcepos: &comrak::nodes::Sourcepos,
    _content: &str,
) -> Option<Token> {
    let token_type = match value {
        NodeValue::Document => "document",
        NodeValue::BlockQuote => "blockQuote",
        NodeValue::List(_) => "list",
        NodeValue::Item(_) => "listItem",
        NodeValue::DescriptionList => "descriptionList",
        NodeValue::DescriptionItem(_) => "descriptionItem",
        NodeValue::DescriptionTerm => "descriptionTerm",
        NodeValue::DescriptionDetails => "descriptionDetails",
        NodeValue::CodeBlock(_) => "codeBlock",
        NodeValue::HtmlBlock(_) => "htmlBlock",
        NodeValue::Paragraph => "paragraph",
        NodeValue::Heading(_) => "heading",
        NodeValue::ThematicBreak => "thematicBreak",
        NodeValue::FootnoteDefinition(_) => "footnoteDefinition",
        NodeValue::Table(_) => "table",
        NodeValue::TableRow(_) => "tableRow",
        NodeValue::TableCell => "tableCell",
        NodeValue::Text(_) => "text",
        NodeValue::TaskItem(_) => "taskItem",
        NodeValue::SoftBreak => "softBreak",
        NodeValue::LineBreak => "lineBreak",
        NodeValue::Code(_) => "code",
        NodeValue::HtmlInline(_) => "htmlInline",
        NodeValue::Emph => "emphasis",
        NodeValue::Strong => "strong",
        NodeValue::Strikethrough => "strikethrough",
        NodeValue::Superscript => "superscript",
        NodeValue::Link(_) => "link",
        NodeValue::Image(_) => "image",
        NodeValue::FootnoteReference(_) => "footnoteReference",
        NodeValue::Math(_) => "math",
        _ => return None,
    };

    Some(Token {
        token_type: token_type.to_string(),
        start_line: sourcepos.start.line,
        start_column: sourcepos.start.column,
        end_line: sourcepos.end.line,
        end_column: sourcepos.end.column,
        text: String::new(), // Would need to extract from content
        children: Vec::new(),
        parent: None,
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
}
