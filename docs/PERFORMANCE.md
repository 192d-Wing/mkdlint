# Performance Analysis

## Current Benchmark Results (v0.10.2)

### Single File Performance

| Benchmark | Time | Notes |
|-----------|------|-------|
| parser_only | 405 µs | ComrakParser parsing of large document (~20KB) |
| lint_single_small | 100 µs | Small document (~100 bytes) |
| lint_single_large | 2.27 ms | Large document (~20KB, 50 sections) |
| lint_realistic_md | 1.12 ms | Realistic doc (~30KB, API docs with code) |
| apply_fixes | 1.55 µs | Apply fixes to fixable errors |

### Multi-File Performance

| Benchmark | Time | Throughput |
|-----------|------|------------|
| lint_multi_20_files | 501 µs | ~40K files/sec |
| lint_multi_100_files | 2.60 ms | ~38K files/sec |

### Auto-Fix Performance

| Benchmark | Time | Notes |
|-----------|------|-------|
| apply_fixes | 1.55 µs | Small document with few fixes |
| apply_fixes_large | ~15 µs | 200 trailing whitespace + 100 proper names |

### Per-Rule Benchmarks

| Benchmark | Time | Notes |
|-----------|------|-------|
| lint_rule_md013 | ~350 µs | 200 lines of varying length |
| lint_rule_md044 | ~400 µs | 200 lines with proper name violations |
| lint_rule_md049_md050 | ~250 µs | 100 lines with emphasis markers |
| lint_none_parser_rules | ~150 µs | Line-based rules only (MD009, MD010, MD013, MD044) |
| lint_micromark_rules | ~800 µs | Parser-dependent rules with headings, lists, emphasis |

### Inline Config Overhead

| Benchmark | Time | Notes |
|-----------|------|-------|
| inline_config/with_directives | ~300 µs | 100 enable/disable directive pairs |
| inline_config/plain | ~200 µs | Same content without directives |

### Configuration Loading

| Benchmark | Time |
|-----------|------|
| config_load_json | 15.9 µs |

## Performance Optimizations Implemented

### 1. PreparedRules (v0.10.1)

**Before**: Each file performed HashMap lookups to check which rules were enabled.
**After**: Rules are filtered once per lint invocation and shared across all files.
**Impact**: ~5-8% improvement on most benchmarks

### 2. Lazy Parser Initialization

**Optimization**: Parser is only invoked if at least one enabled rule requires AST tokens.
**Impact**: ~400µs saved when micromark-based rules are disabled

### 3. Parallel File Linting

**Implementation**: Uses rayon to lint multiple files in parallel.
**Impact**: Near-linear scaling with CPU cores for multi-file workloads

### 4. Zero-Copy Line Splitting

**Optimization**: Lines use `split_inclusive('\n')` which creates string slices, not allocations.
**Impact**: Reduces memory allocations and improves cache locality

## Performance Characteristics

### Scaling Behavior

- **Single file**: Time scales linearly with document size
- **Multi-file**: Parallelizes well across CPU cores (rayon)
- **Memory**: Dominated by ComrakParser AST (~2-3x content size)

### Hot Paths (by profiling)

1. **ComrakParser parsing** (~40% of time for micromark rules)
2. **Regex matching** (~30% of time across all rules)
3. **Line iteration** (~15% of time)
4. **Config lookups** (<5% after PreparedRules optimization)

## Future Optimization Opportunities

### High Impact (>10% improvement potential)

1. **Parser Caching for LSP**
   - Cache parsed AST per document version
   - Only re-parse on content change
   - Estimated impact: 40% faster LSP re-lints

2. **Incremental Linting**
   - Only re-lint changed regions for LSP
   - Track line-based dependencies
   - Estimated impact: 60-80% faster on typical edits

3. **SIMD Line Scanning**
   - Use SIMD for whitespace/newline detection
   - Apply to MD009, MD010, MD047
   - Estimated impact: 20-30% on whitespace rules

### Medium Impact (5-10% improvement)

4. **Regex Compilation Caching**
   - Pre-compile all regexes in rules
   - Use `once_cell::sync::Lazy` consistently
   - Estimated impact: 5-8% on regex-heavy documents

5. **String Interning for Rule Names**
   - Intern rule IDs to avoid string comparisons
   - Use integer comparison in inline config
   - Estimated impact: 2-5% on documents with many directives

### Low Impact (<5% improvement)

6. **Custom Allocator**
   - Try jemalloc/mimalloc for better allocation patterns
   - Estimated impact: 2-4% on large multi-file workloads

7. **Optimize apply_fixes**
   - Pre-allocate output buffer based on content size + fix count
   - Use rope data structure for large documents
   - Estimated impact: 10-20% on fix application (rare operation)

## Comparison with Other Linters

### markdownlint (Node.js original)

**mkdlint advantages**:
- 5-10x faster on small files (no JS startup cost)
- 3-5x faster on large files (native performance)
- Better multi-file scaling (native threads vs Node workers)

**markdownlint advantages**:
- Slightly faster parser (markdown-it is highly optimized)
- Lower memory overhead (V8 GC vs Rust allocator)

### markdownlint-cli2 (Node.js, modern)

**mkdlint advantages**:
- 2-4x faster on single files
- Better LSP performance (no IPC overhead)

**Similar**:
- Multi-file performance (both use parallelism)

## Profiling Instructions

### Using cargo bench

```bash
cargo bench --bench lint_bench
```

### Using Instruments (macOS)

```bash
cargo build --release
instruments -t "Time Profiler" -D /tmp/profile.trace \
  ./target/release/mkdlint test.md
```

### Using perf (Linux)

```bash
cargo build --release
perf record --call-graph dwarf ./target/release/mkdlint test.md
perf report
```

### Using flamegraph

```bash
cargo install flamegraph
cargo flamegraph --bench lint_bench -- --bench
```

## Memory Usage

### Typical Consumption

| Workload | RSS Memory |
|----------|------------|
| Single small file | ~2 MB |
| Single large file (1 MB) | ~8 MB |
| 100 small files | ~15 MB |
| LSP server idle | ~5 MB |
| LSP server with 10 docs | ~12 MB |

### Memory Optimizations

1. **Shared rule registry**: Rules are loaded once, shared across all lints
2. **Config caching**: LSP caches config per directory
3. **Document management**: LSP only keeps open documents in memory
4. **Streaming I/O**: Files are read one at a time, not all into memory

## Benchmark Variance

Typical variance: ±2-5% run-to-run on same hardware
- Parser: Low variance (±2%)
- Regex-heavy rules: Medium variance (±5%)
- Multi-file: Higher variance (±10% due to scheduling)

**Note**: Regressions >15% should be investigated, as they likely indicate a real performance issue rather than noise.
