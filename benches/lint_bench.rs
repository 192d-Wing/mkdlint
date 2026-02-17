use criterion::{Criterion, criterion_group, criterion_main};
use mkdlint::{Config, LintOptions, RuleConfig, apply_fixes, lint_sync};
use std::collections::HashMap;
use std::hint::black_box;

fn generate_small_md() -> String {
    "# Title\n\nSome text here.\n\n## Section\n\nMore text.\n\n### Subsection\n\nFinal text.\n"
        .to_string()
}

fn generate_large_md() -> String {
    let mut content = String::with_capacity(20_000);
    content.push_str("# Large Document\n\n");
    for i in 0..50 {
        content.push_str(&format!("## Section {}\n\n", i));
        for j in 0..10 {
            content.push_str(&format!(
                "This is paragraph {} in section {}. It has some text that makes the line reasonably long.\n\n",
                j, i
            ));
        }
        if i % 5 == 0 {
            content.push_str("```rust\nfn example() {\n    println!(\"hello\");\n}\n```\n\n");
        }
        if i % 3 == 0 {
            content.push_str("- Item one\n- Item two\n- Item three\n\n");
        }
    }
    content
}

fn generate_realistic_md() -> String {
    let mut content = String::with_capacity(30_000);
    content.push_str("# Project Documentation\n\n");
    content.push_str("## Overview\n\n");
    content.push_str(
        "This project provides a comprehensive solution for managing data pipelines.\n\n",
    );
    content.push_str("## Installation\n\n");
    content.push_str("```bash\nnpm install my-package\npip install my-package\n```\n\n");
    content.push_str("## Configuration\n\n");
    content.push_str("| Option | Type | Default | Description |\n");
    content.push_str("|--------|------|---------|-------------|\n");
    for i in 0..20 {
        content.push_str(&format!(
            "| option_{} | string | \"default\" | Description for option {} |\n",
            i, i
        ));
    }
    content.push_str("\n## API Reference\n\n");
    for i in 0..30 {
        content.push_str(&format!("### `function_{}()`\n\n", i));
        content.push_str(&format!(
            "This function performs operation {}. It accepts the following parameters:\n\n",
            i
        ));
        content.push_str(&format!(
            "- `param1` - First parameter for function {}\n",
            i
        ));
        content.push_str(&format!(
            "- `param2` - Second parameter for function {}\n\n",
            i
        ));
        content.push_str(&format!(
            "See [function_{}](#{}) for related functionality.\n\n",
            (i + 1) % 30,
            i
        ));
        if i % 5 == 0 {
            content.push_str("```javascript\nconst result = await myFunction({\n  key: 'value',\n  count: 42\n});\nconsole.log(result);\n```\n\n");
        }
        if i % 7 == 0 {
            content.push_str(
                "> **Note:** This function is deprecated. Use the newer API instead.\n\n",
            );
        }
    }
    content.push_str("## FAQ\n\n");
    for i in 0..10 {
        content.push_str(&format!(
            "**Q: How do I handle case {}?**\n\nA: You should follow these steps:\n\n",
            i
        ));
        content.push_str("1. First step\n2. Second step\n3. Third step\n\n");
    }
    content
}

fn generate_fixable_md() -> String {
    let mut content = String::new();
    content.push_str("# Title\n\n");
    for i in 0..10 {
        content.push_str(&format!("Line {} with trailing whitespace   \n", i));
    }
    content.push_str("\n\n\nExtra blank lines above.\n");
    content.push_str("#missing space\n");
    content.push_str("\thard tab here\n");
    content
}

fn bench_lint_single_small(c: &mut Criterion) {
    let content = generate_small_md();
    c.bench_function("lint_single_small", |b| {
        b.iter(|| {
            let options = LintOptions {
                strings: vec![("bench.md".to_string(), content.clone())]
                    .into_iter()
                    .collect(),
                ..Default::default()
            };
            black_box(lint_sync(&options).unwrap())
        })
    });
}

fn bench_lint_single_large(c: &mut Criterion) {
    let content = generate_large_md();
    c.bench_function("lint_single_large", |b| {
        b.iter(|| {
            let options = LintOptions {
                strings: vec![("bench.md".to_string(), content.clone())]
                    .into_iter()
                    .collect(),
                ..Default::default()
            };
            black_box(lint_sync(&options).unwrap())
        })
    });
}

fn bench_lint_realistic(c: &mut Criterion) {
    let content = generate_realistic_md();
    c.bench_function("lint_realistic_md", |b| {
        b.iter(|| {
            let options = LintOptions {
                strings: vec![("bench.md".to_string(), content.clone())]
                    .into_iter()
                    .collect(),
                ..Default::default()
            };
            black_box(lint_sync(&options).unwrap())
        })
    });
}

fn bench_lint_multi_files(c: &mut Criterion) {
    let content = generate_small_md();
    let strings: HashMap<String, String> = (0..20)
        .map(|i| (format!("file_{}.md", i), content.clone()))
        .collect();

    c.bench_function("lint_multi_20_files", |b| {
        b.iter(|| {
            let options = LintOptions {
                strings: strings.clone(),
                ..Default::default()
            };
            black_box(lint_sync(&options).unwrap())
        })
    });
}

fn bench_lint_multi_100_files(c: &mut Criterion) {
    let content = generate_small_md();
    let strings: HashMap<String, String> = (0..100)
        .map(|i| (format!("file_{}.md", i), content.clone()))
        .collect();

    c.bench_function("lint_multi_100_files", |b| {
        b.iter(|| {
            let options = LintOptions {
                strings: strings.clone(),
                ..Default::default()
            };
            black_box(lint_sync(&options).unwrap())
        })
    });
}

fn bench_apply_fixes(c: &mut Criterion) {
    let content = generate_fixable_md();
    // Lint once to get errors
    let options = LintOptions {
        strings: vec![("bench.md".to_string(), content.clone())]
            .into_iter()
            .collect(),
        ..Default::default()
    };
    let results = lint_sync(&options).unwrap();
    let errors = results.get("bench.md").unwrap();

    c.bench_function("apply_fixes", |b| {
        b.iter(|| black_box(apply_fixes(&content, errors)))
    });
}

fn bench_parser_only(c: &mut Criterion) {
    let content = generate_large_md();
    c.bench_function("parser_only", |b| {
        b.iter(|| black_box(mkdlint::parser::parse(&content)))
    });
}

fn bench_config_load_json(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join(".markdownlint.json");
    let config_json = r#"{
        "default": true,
        "MD001": true,
        "MD003": { "style": "atx" },
        "MD007": { "indent": 4 },
        "MD009": { "br_spaces": 2 },
        "MD013": { "line_length": 120 },
        "MD024": false,
        "MD033": { "allowed_elements": ["br", "hr"] },
        "MD041": true
    }"#;
    std::fs::write(&config_path, config_json).unwrap();

    c.bench_function("config_load_json", |b| {
        b.iter(|| black_box(Config::from_file(&config_path).unwrap()))
    });
}

// ---------------------------------------------------------------------------
// New benchmarks
// ---------------------------------------------------------------------------

fn generate_large_fixable_md() -> String {
    let mut content = String::new();
    content.push_str("# Title\n\n");
    for i in 0..200 {
        content.push_str(&format!("Line {} with trailing whitespace   \n", i));
    }
    for _ in 0..100 {
        content.push_str("Using javascript and github in production.\n");
    }
    content.push_str("#missing space heading\n");
    content
}

fn bench_apply_fixes_large(c: &mut Criterion) {
    let content = generate_large_fixable_md();
    let options = LintOptions {
        strings: vec![("bench.md".to_string(), content.clone())]
            .into_iter()
            .collect(),
        ..Default::default()
    };
    let results = lint_sync(&options).unwrap();
    let errors = results.get("bench.md").unwrap();

    c.bench_function("apply_fixes_large", |b| {
        b.iter(|| black_box(apply_fixes(&content, errors)))
    });
}

fn generate_micromark_heavy_md() -> String {
    let mut content = String::new();
    content.push_str("# Title\n\n");
    for i in 0..30 {
        content.push_str(&format!("## Section {}\n\n", i));
        content.push_str("- Item one\n- Item two\n- Item three\n\n");
        content.push_str(&format!(
            "This has *emphasis* and **strong** and [link {}](url).\n\n",
            i
        ));
    }
    content
}

fn bench_micromark_rules(c: &mut Criterion) {
    let content = generate_micromark_heavy_md();
    c.bench_function("lint_micromark_rules", |b| {
        b.iter(|| {
            let options = LintOptions {
                strings: vec![("bench.md".to_string(), content.clone())]
                    .into_iter()
                    .collect(),
                ..Default::default()
            };
            black_box(lint_sync(&options).unwrap())
        })
    });
}

fn generate_line_scan_md() -> String {
    let mut content = String::new();
    content.push_str("# Title\n\n");
    for i in 0..50 {
        content.push_str(&format!(
            "Line {} with javascript and github in the text   \n",
            i
        ));
    }
    content
}

fn bench_none_parser_rules(c: &mut Criterion) {
    let content = generate_line_scan_md();
    let mut rules = HashMap::new();
    rules.insert("MD009".to_string(), RuleConfig::Enabled(true));
    rules.insert("MD010".to_string(), RuleConfig::Enabled(true));
    rules.insert("MD013".to_string(), RuleConfig::Enabled(true));
    rules.insert("MD044".to_string(), RuleConfig::Enabled(true));
    let config = Config {
        default: Some(false),
        rules,
        ..Default::default()
    };

    c.bench_function("lint_none_parser_rules", |b| {
        b.iter(|| {
            let options = LintOptions {
                strings: vec![("bench.md".to_string(), content.clone())]
                    .into_iter()
                    .collect(),
                config: Some(config.clone()),
                ..Default::default()
            };
            black_box(lint_sync(&options).unwrap())
        })
    });
}

fn bench_rule_md044(c: &mut Criterion) {
    let mut content = String::new();
    content.push_str("# Title\n\n");
    for i in 0..200 {
        content.push_str(&format!(
            "Line {} uses javascript, github, and typescript daily.\n",
            i
        ));
    }

    let mut rules = HashMap::new();
    rules.insert("MD044".to_string(), RuleConfig::Enabled(true));
    let config = Config {
        default: Some(false),
        rules,
        ..Default::default()
    };

    c.bench_function("lint_rule_md044", |b| {
        b.iter(|| {
            let options = LintOptions {
                strings: vec![("bench.md".to_string(), content.clone())]
                    .into_iter()
                    .collect(),
                config: Some(config.clone()),
                ..Default::default()
            };
            black_box(lint_sync(&options).unwrap())
        })
    });
}

fn bench_rule_md013(c: &mut Criterion) {
    let mut content = String::new();
    content.push_str("# Title\n\n");
    for i in 0..200 {
        let line_len = 40 + (i % 100); // Varying lengths from 40 to 139
        let line = "a".repeat(line_len);
        content.push_str(&format!("{}\n", line));
    }

    let mut rules = HashMap::new();
    rules.insert("MD013".to_string(), RuleConfig::Enabled(true));
    let config = Config {
        default: Some(false),
        rules,
        ..Default::default()
    };

    c.bench_function("lint_rule_md013", |b| {
        b.iter(|| {
            let options = LintOptions {
                strings: vec![("bench.md".to_string(), content.clone())]
                    .into_iter()
                    .collect(),
                config: Some(config.clone()),
                ..Default::default()
            };
            black_box(lint_sync(&options).unwrap())
        })
    });
}

fn bench_rule_md049_md050(c: &mut Criterion) {
    let mut content = String::new();
    content.push_str("# Title\n\n");
    for i in 0..100 {
        content.push_str(&format!(
            "Line {} has _emphasis_ and __strong__ markers.\n",
            i
        ));
    }

    let mut rules = HashMap::new();
    rules.insert("MD049".to_string(), RuleConfig::Enabled(true));
    rules.insert("MD050".to_string(), RuleConfig::Enabled(true));
    let config = Config {
        default: Some(false),
        rules,
        ..Default::default()
    };

    c.bench_function("lint_rule_md049_md050", |b| {
        b.iter(|| {
            let options = LintOptions {
                strings: vec![("bench.md".to_string(), content.clone())]
                    .into_iter()
                    .collect(),
                config: Some(config.clone()),
                ..Default::default()
            };
            black_box(lint_sync(&options).unwrap())
        })
    });
}

fn bench_inline_config(c: &mut Criterion) {
    let mut content_with_directives = String::new();
    content_with_directives.push_str("# Title\n\n");
    for i in 0..100 {
        content_with_directives.push_str(&format!(
            "<!-- markdownlint-disable MD013 -->\nLine {} is here.\n<!-- markdownlint-enable MD013 -->\n",
            i
        ));
    }

    let mut content_plain = String::new();
    content_plain.push_str("# Title\n\n");
    for i in 0..100 {
        content_plain.push_str(&format!("Line {} is here.\n", i));
    }

    let mut group = c.benchmark_group("inline_config_overhead");
    group.bench_function("with_directives", |b| {
        b.iter(|| {
            let options = LintOptions {
                strings: vec![("bench.md".to_string(), content_with_directives.clone())]
                    .into_iter()
                    .collect(),
                ..Default::default()
            };
            black_box(lint_sync(&options).unwrap())
        })
    });
    group.bench_function("plain", |b| {
        b.iter(|| {
            let options = LintOptions {
                strings: vec![("bench.md".to_string(), content_plain.clone())]
                    .into_iter()
                    .collect(),
                ..Default::default()
            };
            black_box(lint_sync(&options).unwrap())
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_parser_only,
    bench_lint_single_small,
    bench_lint_single_large,
    bench_lint_realistic,
    bench_lint_multi_files,
    bench_lint_multi_100_files,
    bench_apply_fixes,
    bench_config_load_json,
    bench_apply_fixes_large,
    bench_micromark_rules,
    bench_none_parser_rules,
    bench_rule_md044,
    bench_rule_md013,
    bench_rule_md049_md050,
    bench_inline_config,
);
criterion_main!(benches);
