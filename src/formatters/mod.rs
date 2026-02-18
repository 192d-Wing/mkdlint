//! Output formatters for lint results

mod github;
mod json;
mod sarif;
mod text;

pub use github::format_github;
pub use json::format_json;
pub use sarif::format_sarif;
pub use text::{format_text, format_text_with_context};
