//! File expansion and ignore-pattern filtering

/// Expand directories to .md/.markdown files recursively
pub(crate) fn expand_paths(paths: &[String]) -> Vec<String> {
    use walkdir::WalkDir;

    let mut expanded = Vec::new();
    for path in paths {
        let p = std::path::Path::new(path);
        if p.is_dir() {
            for entry in WalkDir::new(p).into_iter().filter_map(|e| e.ok()) {
                let ep = entry.path();
                if ep.is_file()
                    && let Some(ext) = ep.extension().and_then(|e| e.to_str())
                    && (ext == "md" || ext == "markdown")
                {
                    expanded.push(ep.to_string_lossy().to_string());
                }
            }
        } else {
            expanded.push(path.clone());
        }
    }
    expanded.sort();
    expanded
}

/// Filter files by ignore glob patterns
pub(crate) fn filter_ignored(
    files: Vec<String>,
    ignore_patterns: &[String],
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if ignore_patterns.is_empty() {
        return Ok(files);
    }

    use globset::{Glob, GlobSetBuilder};

    let mut builder = GlobSetBuilder::new();
    for pattern in ignore_patterns {
        builder.add(Glob::new(pattern)?);
    }
    let ignore_set = builder.build()?;

    Ok(files
        .into_iter()
        .filter(|f| !ignore_set.is_match(f))
        .collect())
}
