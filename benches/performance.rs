//! Performance benchmarks for diff-gitignore-filter
//!
//! Measures processing speed and memory usage with various input sizes
//! and complexity levels to ensure O(1) memory usage and acceptable
//! processing times.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use diff_gitignore_filter::Filter;
use std::fs;
use std::hint::black_box;
use std::io::Cursor;
use tempfile::TempDir;

/// Create a test repository with specified .gitignore content
fn create_benchmark_repo(gitignore_content: &str) -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create .git directory
    fs::create_dir(temp_dir.path().join(".git")).expect("Failed to create .git");

    // Create .gitignore
    fs::write(temp_dir.path().join(".gitignore"), gitignore_content)
        .expect("Failed to write .gitignore");

    temp_dir
}

/// Generate a diff with specified number of files
fn generate_diff(num_files: usize, ignored_ratio: f32) -> String {
    let mut diff = String::new();

    for i in 0..num_files {
        let filename = if (i as f32 / num_files as f32) < ignored_ratio {
            format!("debug_{i}.log") // Will be ignored
        } else {
            format!("src/file_{i}.rs") // Will not be ignored
        };

        diff.push_str(&format!(
            "diff --git a/{} b/{}\n\
             index {}..{} 100644\n\
             --- a/{}\n\
             +++ b/{}\n\
             @@ -1,3 +1,4 @@\n\
             fn main() {{\n\
             +    println!(\"Hello from file {}\");\n\
             }}\n",
            filename,
            filename,
            i,
            i + 1000,
            filename,
            filename,
            i
        ));
    }

    diff
}

/// Benchmark basic diff processing with various input sizes
fn bench_diff_processing(c: &mut Criterion) {
    let temp_dir = create_benchmark_repo("*.log\ntarget/\n");
    let filter = Filter::new(temp_dir.path()).expect("Failed to create filter");

    let mut group = c.benchmark_group("diff_processing");

    for size in [10, 100, 1000, 5000].iter() {
        let diff_content = generate_diff(*size, 0.3); // 30% ignored files

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("files", size), &diff_content, |b, diff| {
            b.iter(|| {
                let mut output = Vec::new();
                filter
                    .process_diff(Cursor::new(black_box(diff)), &mut output)
                    .expect("Processing failed");
                black_box(output);
            });
        });
    }

    group.finish();
}

/// Benchmark with different ignore ratios
fn bench_ignore_ratios(c: &mut Criterion) {
    let temp_dir = create_benchmark_repo("*.log\n*.tmp\ntarget/\n");
    let filter = Filter::new(temp_dir.path()).expect("Failed to create filter");

    let mut group = c.benchmark_group("ignore_ratios");

    for ratio in [0.0, 0.25, 0.5, 0.75, 1.0].iter() {
        let diff_content = generate_diff(1000, *ratio);

        group.bench_with_input(
            BenchmarkId::new("ratio", (ratio * 100.0) as u32),
            &diff_content,
            |b, diff| {
                b.iter(|| {
                    let mut output = Vec::new();
                    filter
                        .process_diff(Cursor::new(black_box(diff)), &mut output)
                        .expect("Processing failed");
                    black_box(output);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark filter creation with complex .gitignore patterns
fn bench_filter_creation(c: &mut Criterion) {
    let simple_gitignore = "*.log\n*.tmp\n";
    let complex_gitignore = include_str!("../tests/fixtures/complex_gitignore.txt");

    let mut group = c.benchmark_group("filter_creation");

    group.bench_function("simple_gitignore", |b| {
        b.iter(|| {
            let temp_dir = create_benchmark_repo(black_box(simple_gitignore));
            let filter = Filter::new(temp_dir.path()).expect("Failed to create filter");
            black_box(filter);
        });
    });

    group.bench_function("complex_gitignore", |b| {
        b.iter(|| {
            let temp_dir = create_benchmark_repo(black_box(complex_gitignore));
            let filter = Filter::new(temp_dir.path()).expect("Failed to create filter");
            black_box(filter);
        });
    });

    group.finish();
}

/// Benchmark memory usage with large inputs
fn bench_memory_usage(c: &mut Criterion) {
    let temp_dir = create_benchmark_repo("*.log\n");
    let filter = Filter::new(temp_dir.path()).expect("Failed to create filter");

    let mut group = c.benchmark_group("memory_usage");

    // Test with very large diffs to ensure O(1) memory usage
    for size in [10000, 50000, 100000].iter() {
        let diff_content = generate_diff(*size, 0.5);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("large_diff", size),
            &diff_content,
            |b, diff| {
                b.iter(|| {
                    let mut output = Vec::new();
                    filter
                        .process_diff(Cursor::new(black_box(diff)), &mut output)
                        .expect("Processing failed");
                    black_box(output);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark pattern matching performance
fn bench_pattern_matching(c: &mut Criterion) {
    let temp_dir = create_benchmark_repo("*.log\n*.tmp\n*.cache\n*.build\n*.test\n");
    let filter = Filter::new(temp_dir.path()).expect("Failed to create filter");

    let mut group = c.benchmark_group("pattern_matching");

    // Create diff with many different file extensions
    let mut diff = String::new();
    let extensions = [
        "rs", "log", "tmp", "cache", "build", "test", "md", "txt", "json", "toml",
    ];

    for i in 0..1000 {
        let ext = extensions[i % extensions.len()];
        let filename = format!("file_{i}.{ext}");

        diff.push_str(&format!(
            "diff --git a/{} b/{}\n\
             index {}..{} 100644\n\
             content line\n",
            filename,
            filename,
            i,
            i + 1000
        ));
    }

    group.bench_function("mixed_extensions", |b| {
        b.iter(|| {
            let mut output = Vec::new();
            filter
                .process_diff(Cursor::new(black_box(&diff)), &mut output)
                .expect("Processing failed");
            black_box(output);
        });
    });

    group.finish();
}

/// Benchmark realistic Git repository scenarios
fn bench_realistic_scenarios(c: &mut Criterion) {
    let temp_dir = create_benchmark_repo(include_str!("../tests/fixtures/complex_gitignore.txt"));
    let filter = Filter::new(temp_dir.path()).expect("Failed to create filter");

    let mut group = c.benchmark_group("realistic_scenarios");

    // Simulate typical development workflow diffs
    let scenarios = vec![
        ("small_feature", generate_realistic_diff(5, "feature")),
        ("medium_refactor", generate_realistic_diff(25, "refactor")),
        ("large_merge", generate_realistic_diff(100, "merge")),
    ];

    for (name, diff_content) in scenarios {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut output = Vec::new();
                filter
                    .process_diff(Cursor::new(black_box(&diff_content)), &mut output)
                    .expect("Processing failed");
                black_box(output);
            });
        });
    }

    group.finish();
}

/// Generate realistic diff content for benchmarking
fn generate_realistic_diff(num_files: usize, scenario: &str) -> String {
    let mut diff = String::new();

    let file_types = match scenario {
        "feature" => vec![
            "src/lib.rs",
            "src/feature.rs",
            "tests/feature_test.rs",
            "README.md",
        ],
        "refactor" => vec![
            "src/main.rs",
            "src/lib.rs",
            "src/utils.rs",
            "src/config.rs",
            "Cargo.toml",
        ],
        "merge" => vec!["src/", "tests/", "docs/", "examples/", "benches/"],
        _ => vec!["src/file.rs"],
    };

    for i in 0..num_files {
        let base_path = file_types[i % file_types.len()];
        let filename = if base_path.ends_with('/') {
            format!("{base_path}file_{i}.rs")
        } else {
            base_path.to_string()
        };

        diff.push_str(&format!(
            "diff --git a/{} b/{}\n\
             index {}..{} 100644\n\
             --- a/{}\n\
             +++ b/{}\n\
             @@ -10,6 +10,7 @@\n\
             // Existing code\n\
             fn existing_function() {{\n\
             +    // Added in {}\n\
                 println!(\"Hello\");\n\
             }}\n\
             \n",
            filename,
            filename,
            i,
            i + 1000,
            filename,
            filename,
            scenario
        ));
    }

    diff
}

criterion_group!(
    benches,
    bench_diff_processing,
    bench_ignore_ratios,
    bench_filter_creation,
    bench_memory_usage,
    bench_pattern_matching,
    bench_realistic_scenarios
);

criterion_main!(benches);
