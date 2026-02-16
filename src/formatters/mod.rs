//! Output formatters for lint results

mod json;
mod sarif;
mod text;

pub use json::format_json;
pub use sarif::format_sarif;
pub use text::format_text;
