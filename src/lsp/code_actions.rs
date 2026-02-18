//! Convert mkdlint fix_info to LSP code actions

use crate::types::LintError;
use std::collections::HashMap;

use super::utils::to_position;

// Import all LSP types from tower-lsp which re-exports lsp-types
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Diagnostic, Position, Range, TextEdit, Url,
    WorkspaceEdit,
};

/// Convert a LintError with fix_info to a CodeAction.
///
/// If `diagnostic` is provided, the action will reference it so the editor
/// can show a lightbulb specifically for that diagnostic.
pub fn fix_to_code_action(
    uri: &Url,
    error: &LintError,
    content: &str,
    diagnostic: Option<Diagnostic>,
) -> Option<CodeActionOrCommand> {
    let fix_info = error.fix_info.as_ref()?;

    let text_edit = calculate_text_edit(error, fix_info, content)?;

    let mut changes = HashMap::new();
    changes.insert(uri.clone(), vec![text_edit]);

    let workspace_edit = WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    };

    let title = format!(
        "Fix: {} ({})",
        error.rule_description,
        error.rule_names.first().unwrap_or(&"unknown")
    );

    let code_action = CodeAction {
        title,
        kind: Some(CodeActionKind::QUICKFIX),
        edit: Some(workspace_edit),
        diagnostics: diagnostic.map(|d| vec![d]),
        ..Default::default()
    };

    Some(CodeActionOrCommand::CodeAction(code_action))
}

/// Calculate the TextEdit from FixInfo
fn calculate_text_edit(
    error: &LintError,
    fix_info: &crate::types::FixInfo,
    content: &str,
) -> Option<TextEdit> {
    let lines: Vec<&str> = content.lines().collect();

    // Determine target line
    let target_line = fix_info.line_number.unwrap_or(error.line_number);

    let line_idx = target_line.saturating_sub(1);
    let _line = lines.get(line_idx)?;

    // Handle delete entire line case
    if fix_info.delete_count == Some(-1) {
        return Some(create_delete_line_edit(target_line, lines.len()));
    }

    // Get edit column (1-based)
    let edit_col = fix_info.edit_column?;

    // Calculate start position
    let start = to_position(target_line, edit_col);

    // Calculate end position based on delete_count
    let end = if let Some(delete_count) = fix_info.delete_count {
        if delete_count > 0 {
            Position {
                line: start.line,
                character: start.character + delete_count as u32,
            }
        } else {
            start // delete_count == 0 means insert only
        }
    } else {
        start // No deletion, just insertion
    };

    let range = Range { start, end };
    let new_text = fix_info.insert_text.clone().unwrap_or_default();

    Some(TextEdit { range, new_text })
}

/// Create a TextEdit that deletes an entire line (including newline)
fn create_delete_line_edit(line_number: usize, total_lines: usize) -> TextEdit {
    let line_idx = line_number.saturating_sub(1);

    // Delete the entire line including newline
    let start = Position {
        line: line_idx as u32,
        character: 0,
    };

    // If this is not the last line, delete up to start of next line
    // If it is the last line, delete to end of line
    let end = if line_number < total_lines {
        Position {
            line: (line_idx + 1) as u32,
            character: 0,
        }
    } else {
        Position {
            line: line_idx as u32,
            character: u32::MAX, // Delete to end of line
        }
    };

    TextEdit {
        range: Range { start, end },
        new_text: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FixInfo, Severity};

    fn create_test_error_with_fix(fix_info: FixInfo) -> LintError {
        LintError {
            line_number: 1,
            rule_names: &["MD001"],
            rule_description: "Test rule",
            error_detail: None,
            error_context: None,
            rule_information: None,
            error_range: None,
            fix_info: Some(fix_info),
            suggestion: Some("Apply fix".to_string()),
            severity: Severity::Error,
            fix_only: false,
        }
    }

    #[test]
    fn test_insert_text_fix() {
        let fix_info = FixInfo {
            line_number: None,
            edit_column: Some(3),
            delete_count: None,
            insert_text: Some(" ".to_string()),
        };

        let error = create_test_error_with_fix(fix_info);
        let content = "# Test\n";
        let uri = Url::parse("file:///tmp/test.md").unwrap();

        let action = fix_to_code_action(&uri, &error, content, None);
        assert!(action.is_some());

        if let Some(CodeActionOrCommand::CodeAction(ca)) = action {
            assert_eq!(ca.kind, Some(CodeActionKind::QUICKFIX));
            assert!(ca.title.contains("Test rule"));

            let edit = ca.edit.unwrap();
            let changes = edit.changes.unwrap();
            let text_edits = changes.get(&uri).unwrap();
            assert_eq!(text_edits.len(), 1);

            let text_edit = &text_edits[0];
            assert_eq!(text_edit.range.start, Position::new(0, 2));
            assert_eq!(text_edit.range.end, Position::new(0, 2));
            assert_eq!(text_edit.new_text, " ");
        }
    }

    #[test]
    fn test_delete_chars_fix() {
        let fix_info = FixInfo {
            line_number: None,
            edit_column: Some(3),
            delete_count: Some(2),
            insert_text: None,
        };

        let error = create_test_error_with_fix(fix_info);
        let content = "#  Test\n"; // Two spaces
        let uri = Url::parse("file:///tmp/test.md").unwrap();

        let action = fix_to_code_action(&uri, &error, content, None);
        assert!(action.is_some());

        if let Some(CodeActionOrCommand::CodeAction(ca)) = action {
            let edit = ca.edit.unwrap();
            let changes = edit.changes.unwrap();
            let text_edits = changes.get(&uri).unwrap();
            let text_edit = &text_edits[0];

            assert_eq!(text_edit.range.start, Position::new(0, 2));
            assert_eq!(text_edit.range.end, Position::new(0, 4));
            assert_eq!(text_edit.new_text, "");
        }
    }

    #[test]
    fn test_replace_text_fix() {
        let fix_info = FixInfo {
            line_number: None,
            edit_column: Some(1),
            delete_count: Some(9),
            insert_text: Some("## Heading".to_string()),
        };

        let error = create_test_error_with_fix(fix_info);
        let content = "_Heading_\n";
        let uri = Url::parse("file:///tmp/test.md").unwrap();

        let action = fix_to_code_action(&uri, &error, content, None);
        assert!(action.is_some());

        if let Some(CodeActionOrCommand::CodeAction(ca)) = action {
            let edit = ca.edit.unwrap();
            let changes = edit.changes.unwrap();
            let text_edits = changes.get(&uri).unwrap();
            let text_edit = &text_edits[0];

            assert_eq!(text_edit.range.start, Position::new(0, 0));
            assert_eq!(text_edit.range.end, Position::new(0, 9));
            assert_eq!(text_edit.new_text, "## Heading");
        }
    }

    #[test]
    fn test_delete_line_fix() {
        let fix_info = FixInfo {
            line_number: Some(2),
            edit_column: Some(1),
            delete_count: Some(-1),
            insert_text: None,
        };

        let error = create_test_error_with_fix(fix_info);
        let content = "> line 1\n\n> line 2\n";
        let uri = Url::parse("file:///tmp/test.md").unwrap();

        let action = fix_to_code_action(&uri, &error, content, None);
        assert!(action.is_some());

        if let Some(CodeActionOrCommand::CodeAction(ca)) = action {
            let edit = ca.edit.unwrap();
            let changes = edit.changes.unwrap();
            let text_edits = changes.get(&uri).unwrap();
            let text_edit = &text_edits[0];

            // Should delete line 2 (index 1) up to start of line 3
            assert_eq!(text_edit.range.start, Position::new(1, 0));
            assert_eq!(text_edit.range.end, Position::new(2, 0));
            assert_eq!(text_edit.new_text, "");
        }
    }

    #[test]
    fn test_no_fix_info() {
        let mut error = create_test_error_with_fix(FixInfo {
            line_number: None,
            edit_column: None,
            delete_count: None,
            insert_text: None,
        });
        error.fix_info = None;

        let content = "# Test\n";
        let uri = Url::parse("file:///tmp/test.md").unwrap();

        let action = fix_to_code_action(&uri, &error, content, None);
        assert!(action.is_none());
    }
}
