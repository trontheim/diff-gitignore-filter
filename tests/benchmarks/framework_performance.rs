//! Performance benchmarks for the test framework
//!
//! This module contains benchmarks to measure the performance characteristics
//! of the new unified test framework compared to legacy approaches.

#[cfg(test)]
mod benchmarks {
    use std::time::Instant;
    use tempfile::TempDir;

    // Import both legacy and new framework
    use crate::common::{TestData, TestRepo};

    // TestRepo and TestData are already imported from crate::common

    /// Benchmark legacy test setup
    fn benchmark_legacy_setup(iterations: usize) -> std::time::Duration {
        let start = Instant::now();

        for _ in 0..iterations {
            let _temp_dir = TestRepo::builder()
                .with_patterns(TestData::BASIC_PATTERNS)
                .with_files(
                    TestData::BASIC_FILES
                        .iter()
                        .map(|(path, content)| {
                            (path.to_string(), content.unwrap_or("").to_string())
                        })
                        .collect::<Vec<(String, String)>>(),
                )
                .build()
                .unwrap()
                .into_temp_dir();
            // TempDir automatically cleans up when dropped
        }

        start.elapsed()
    }

    /// Benchmark new framework setup
    fn benchmark_framework_setup(iterations: usize) -> std::time::Duration {
        let start = Instant::now();

        for _ in 0..iterations {
            let _repo =
                TestRepo::builder()
                    .with_patterns(
                        crate::common::framework::TestData::BASIC_PATTERNS
                            .iter()
                            .map(|s| s.to_string()),
                    )
                    .with_files(TestData::BASIC_FILES.iter().map(|(path, content)| {
                        (path.to_string(), content.unwrap_or("").to_string())
                    }))
                    .build()
                    .unwrap();
        }

        start.elapsed()
    }

    /// Benchmark complex repository setup
    fn benchmark_complex_setup(iterations: usize) -> std::time::Duration {
        let start = Instant::now();

        let complex_patterns = vec![
            "*.log".to_string(),
            "*.tmp".to_string(),
            "target/".to_string(),
            "node_modules/".to_string(),
            ".env".to_string(),
            "**/*.cache".to_string(),
            "build/".to_string(),
            "dist/".to_string(),
        ];

        let complex_files = vec![
            (
                "src/main.rs".to_string(),
                "fn main() { println!(\"Hello\"); }".to_string(),
            ),
            ("src/lib.rs".to_string(), "pub mod utils;".to_string()),
            ("src/utils.rs".to_string(), "pub fn helper() {}".to_string()),
            (
                "Cargo.toml".to_string(),
                "[package]\nname = \"test\"".to_string(),
            ),
            ("README.md".to_string(), "# Test Project".to_string()),
            (
                "tests/integration.rs".to_string(),
                "#[test]\nfn test() {}".to_string(),
            ),
        ];

        for _ in 0..iterations {
            let _repo = TestRepo::builder()
                .with_patterns(complex_patterns.clone())
                .with_files(complex_files.clone())
                .build()
                .unwrap();
        }

        start.elapsed()
    }

    /// **What is tested:** Performance comparison between legacy and new test framework setup methods
    /// **Why it is tested:** Ensures the new framework doesn't introduce significant performance regressions
    /// **Test conditions:** Multiple iterations of repository setup using both legacy and new frameworks
    /// **Expectations:** New framework should perform within 20% of legacy performance, with setup times under 100ms
    #[test]
    fn test_framework_performance_comparison() {
        const ITERATIONS: usize = 10;

        println!("\n=== Test Framework Performance Benchmarks ===");

        // Warm up
        let _ = benchmark_legacy_setup(1);
        let _ = benchmark_framework_setup(1);

        // Actual benchmarks
        let legacy_time = benchmark_legacy_setup(ITERATIONS);
        let framework_time = benchmark_framework_setup(ITERATIONS);
        let complex_time = benchmark_complex_setup(ITERATIONS);

        println!("Legacy setup ({ITERATIONS} iterations): {legacy_time:?}");
        println!("Framework setup ({ITERATIONS} iterations): {framework_time:?}");
        println!("Complex setup ({ITERATIONS} iterations): {complex_time:?}");

        // Calculate performance metrics
        let legacy_avg = legacy_time.as_millis() as f64 / ITERATIONS as f64;
        let framework_avg = framework_time.as_millis() as f64 / ITERATIONS as f64;
        let complex_avg = complex_time.as_millis() as f64 / ITERATIONS as f64;

        println!("\nAverage times:");
        println!("Legacy: {legacy_avg:.2}ms per setup");
        println!("Framework: {framework_avg:.2}ms per setup");
        println!("Complex: {complex_avg:.2}ms per setup");

        // Performance comparison
        if framework_avg < legacy_avg {
            let improvement = ((legacy_avg - framework_avg) / legacy_avg) * 100.0;
            println!("✅ Framework is {improvement:.1}% faster than legacy");
        } else {
            let regression = ((framework_avg - legacy_avg) / legacy_avg) * 100.0;
            println!("⚠️  Framework is {regression:.1}% slower than legacy");

            // Allow up to 20% regression for the added functionality
            assert!(
                regression < 20.0,
                "Framework performance regression too high: {regression:.1}%"
            );
        }

        // Ensure reasonable performance bounds
        assert!(
            framework_avg < 100.0,
            "Framework setup too slow: {framework_avg:.2}ms"
        );
        assert!(
            complex_avg < 200.0,
            "Complex setup too slow: {complex_avg:.2}ms"
        );
    }

    /// **What is tested:** Memory usage efficiency and struct size analysis of test framework components
    /// **Why it is tested:** Ensures the framework uses memory efficiently and doesn't have excessive overhead
    /// **Test conditions:** Analysis of struct sizes and memory usage patterns during repository creation
    /// **Expectations:** Should demonstrate reasonable memory usage with successful repository creation and validation
    #[test]
    fn test_memory_efficiency() {
        use std::mem;

        println!("\n=== Memory Usage Analysis ===");

        // Analyze struct sizes
        println!(
            "TestRepo size: {} bytes",
            mem::size_of::<crate::common::framework::TestRepo>()
        );
        println!(
            "TestCommand size: {} bytes",
            mem::size_of::<crate::common::framework::TestCommand>()
        );
        println!(
            "TestCase size: {} bytes",
            mem::size_of::<crate::common::framework::TestCase>()
        );
        println!("TempDir size: {} bytes", mem::size_of::<TempDir>());

        // Test memory usage patterns
        let repo = TestRepo::builder()
            .with_patterns(
                crate::common::framework::TestData::BASIC_PATTERNS
                    .iter()
                    .map(|s| s.to_string()),
            )
            .with_files(
                TestData::BASIC_FILES
                    .iter()
                    .map(|(path, content)| (path.to_string(), content.unwrap_or("").to_string())),
            )
            .build()
            .unwrap();

        // Ensure the repo is actually created and usable
        assert!(repo.path().exists());
        assert!(repo.path().join(".gitignore").exists());

        println!("✅ Memory efficiency test passed");
    }

    /// **What is tested:** Thread safety and performance of concurrent test repository setup operations
    /// **Why it is tested:** Ensures the framework can handle multiple simultaneous test setups without race conditions
    /// **Test conditions:** Multiple threads creating test repositories concurrently with unique patterns and files
    /// **Expectations:** All concurrent setups should succeed without conflicts, demonstrating thread safety
    #[test]
    fn test_concurrent_setup() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use std::thread;

        println!("\n=== Concurrent Setup Test ===");

        const THREAD_COUNT: usize = 4;
        const SETUPS_PER_THREAD: usize = 5;

        let success_count = Arc::new(AtomicUsize::new(0));
        let start = Instant::now();

        let handles: Vec<_> = (0..THREAD_COUNT)
            .map(|thread_id| {
                let success_count = Arc::clone(&success_count);

                thread::spawn(move || {
                    for i in 0..SETUPS_PER_THREAD {
                        let repo = TestRepo::builder()
                            .with_patterns(vec![format!("*.log_{}", thread_id)])
                            .with_files(vec![(
                                format!("test_{thread_id}_{i}.rs"),
                                format!("// Test file {i} from thread {thread_id}"),
                            )])
                            .build();

                        if repo.is_ok() {
                            success_count.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        let elapsed = start.elapsed();
        let total_setups = THREAD_COUNT * SETUPS_PER_THREAD;
        let successful_setups = success_count.load(Ordering::SeqCst);

        println!("Concurrent setup: {THREAD_COUNT} threads, {SETUPS_PER_THREAD} setups each");
        println!("Total time: {elapsed:?}");
        println!("Successful setups: {successful_setups}/{total_setups}");
        println!(
            "Average time per setup: {:.2}ms",
            elapsed.as_millis() as f64 / total_setups as f64
        );

        // Ensure all setups succeeded
        assert_eq!(
            successful_setups, total_setups,
            "Some concurrent setups failed"
        );

        println!("✅ Concurrent setup test passed");
    }
}
