//! Test utilities for Plantastic.
//!
//! Provides timeout enforcement for tests. In this project, Rust code is
//! compute-bound and fast. A test taking >10 seconds is almost certainly
//! blocked on I/O (database, network, subprocess) and should either:
//!
//! - Be restructured to not block
//! - Use `run_with_timeout` with an explicit, justified duration
//! - Be marked `#[ignore]` with a scenario ID explaining why

use std::time::{Duration, Instant};

/// Default test timeout. Any test exceeding this is suspect.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Run a closure with a timeout. Panics if the closure takes longer than
/// the specified duration. The panic message includes timing information
/// to help diagnose what's slow.
///
/// Note: this does NOT interrupt the closure — it checks elapsed time
/// after the closure returns. For truly hung operations, the test binary
/// timeout in the justfile is the backstop.
///
/// # Panics
///
/// Panics if `f` takes longer than `timeout` to execute, or if elapsed
/// time exceeds 50% of the timeout (warning only, via stderr).
///
/// # Usage
///
/// ```rust,ignore
/// use pt_test_utils::run_with_timeout;
/// use std::time::Duration;
///
/// #[test]
/// fn my_test() {
///     run_with_timeout(Duration::from_secs(5), || {
///         // test body
///     });
/// }
/// ```
pub fn run_with_timeout<F, R>(timeout: Duration, f: F) -> R
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed();

    assert!(
        elapsed <= timeout,
        "SLOW TEST: took {:.2}s (timeout: {:.2}s). \
         Rust compute should not take this long — \
         check for I/O waits, unnecessary allocations, \
         or missing test data setup.",
        elapsed.as_secs_f64(),
        timeout.as_secs_f64()
    );

    if elapsed > timeout / 2 {
        eprintln!(
            "  WARNING: test took {:.2}s ({:.0}% of {:.0}s timeout)",
            elapsed.as_secs_f64(),
            (elapsed.as_secs_f64() / timeout.as_secs_f64()) * 100.0,
            timeout.as_secs_f64()
        );
    }

    result
}

/// Convenience: run with the default 10-second timeout.
///
/// # Panics
///
/// Panics if `f` takes longer than 10 seconds.
pub fn timed<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    run_with_timeout(DEFAULT_TIMEOUT, f)
}
