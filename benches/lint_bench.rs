use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use mdlint::{lint_sync, apply_fixes, LintOptions};
use std::collections::HashMap;

fn generate_small_md() -> String {
    "# Title\n\nSome text here.\n\n## Section\n\nMore text.\n\n### Subsection\n\nFinal text.\n".to_string()
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
        b.iter(|| {
            black_box(apply_fixes(&content, errors))
        })
    });
}

fn bench_parser_only(c: &mut Criterion) {
    let content = generate_large_md();
    c.bench_function("parser_only", |b| {
        b.iter(|| {
            black_box(mdlint::parser::parse(&content))
        })
    });
}

criterion_group!(
    benches,
    bench_parser_only,
    bench_lint_single_small,
    bench_lint_single_large,
    bench_lint_multi_files,
    bench_apply_fixes,
);
criterion_main!(benches);
