//! Database connection pool with retry logic, tuned for AWS Lambda + Neon serverless.
//!
//! Neon's serverless Postgres can cold-start when Lambda also cold-starts, causing
//! connection hangs. This module provides configurable timeouts and exponential-backoff
//! retry on transient connection failures.
//!
//! ## Connection string support
//!
//! sqlx parses these query parameters automatically from `DATABASE_URL`:
//!
//! - `sslmode=require` — required for Neon
//! - `sslnegotiation=direct` — skips SSLRequest round-trip, ~50-100ms faster
//! - `statement_cache_size=0` — required for Neon pooled connections (PgBouncer transaction mode)
//!
//! Neon pooled connections use the `-pooler` hostname suffix
//! (e.g., `ep-cool-name-pooler.us-west-2.aws.neon.tech`). No special code handling
//! is needed — it's just a different DNS name.

use std::str::FromStr;
use std::time::Duration;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;

use crate::error::RepoError;

/// Pool configuration with Lambda-tuned defaults.
///
/// All timeouts and limits are configurable for different environments:
/// - Lambda: low `max_connections`, longer `connect_timeout` for Neon cold-starts
/// - Local dev: defaults work fine, connection is immediate
/// - Tests: use short timeouts and zero retries for fast failure
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Timeout for each pool creation attempt. Covers TCP connect + TLS + auth.
    /// Neon cold-starts can take 3-8s; default 15s gives headroom.
    pub connect_timeout: Duration,
    /// How long to wait for a connection from the pool after it's established.
    pub acquire_timeout: Duration,
    /// Drop idle connections after this duration. Lambda freezes between invocations.
    pub idle_timeout: Duration,
    /// Maximum connections in the pool. Lambda has limited concurrency per instance.
    pub max_connections: u32,
    /// Minimum connections kept alive. 0 allows scale-to-zero between Lambda invocations.
    pub min_connections: u32,
    /// Number of connection attempts before giving up (1 = no retries).
    pub max_retries: u32,
    /// Initial backoff between retries. Doubles on each subsequent attempt.
    pub initial_backoff: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(15),
            acquire_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(30),
            max_connections: 5,
            min_connections: 0,
            max_retries: 3,
            initial_backoff: Duration::from_millis(500),
        }
    }
}

/// Create a connection pool with default Lambda-tuned configuration.
///
/// Equivalent to `create_pool_with_config(database_url, &PoolConfig::default())`.
pub async fn create_pool(database_url: &str) -> Result<PgPool, RepoError> {
    create_pool_with_config(database_url, &PoolConfig::default()).await
}

/// Create a connection pool with explicit configuration.
///
/// Parses `database_url` into connection options, applies `config` timeouts,
/// and retries on transient failures with exponential backoff.
pub async fn create_pool_with_config(
    database_url: &str,
    config: &PoolConfig,
) -> Result<PgPool, RepoError> {
    tracing::debug!(?config, "Creating database pool");

    let connect_opts = PgConnectOptions::from_str(database_url).map_err(RepoError::Database)?;

    let pool_opts = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .idle_timeout(config.idle_timeout)
        .acquire_timeout(config.acquire_timeout);

    connect_with_retry(pool_opts, connect_opts, config)
        .await
        .map_err(RepoError::Database)
}

/// Classify whether a sqlx error is transient (worth retrying).
///
/// Transient: I/O failures (TCP, DNS, timeout), TLS handshake, pool timeout.
/// Permanent: configuration errors, authentication, server-side errors.
fn is_transient(err: &sqlx::Error) -> bool {
    matches!(
        err,
        sqlx::Error::Io(_) | sqlx::Error::Tls(_) | sqlx::Error::PoolTimedOut
    )
}

/// Saturating cast from u128 millis to u64 for tracing fields.
/// Connection durations are always well within u64 range.
#[allow(clippy::cast_possible_truncation)]
fn millis(d: Duration) -> u64 {
    d.as_millis() as u64
}

/// Attempt pool connection with exponential backoff on transient failures.
///
/// Each attempt is wrapped in a `tokio::time::timeout` using `config.connect_timeout`.
/// If the timeout fires, the attempt is treated as a transient I/O error.
async fn connect_with_retry(
    pool_opts: PgPoolOptions,
    connect_opts: PgConnectOptions,
    config: &PoolConfig,
) -> Result<PgPool, sqlx::Error> {
    let start = std::time::Instant::now();
    let mut backoff = config.initial_backoff;
    let mut last_err = None;

    for attempt in 1..=config.max_retries {
        let result = tokio::time::timeout(
            config.connect_timeout,
            pool_opts.clone().connect_with(connect_opts.clone()),
        )
        .await;

        match result {
            Ok(Ok(pool)) => {
                let elapsed = start.elapsed();
                if attempt > 1 {
                    tracing::info!(
                        attempt,
                        total_attempts = config.max_retries,
                        elapsed_ms = millis(elapsed),
                        "Database pool connected after retries"
                    );
                } else {
                    tracing::info!(elapsed_ms = millis(elapsed), "Database pool connected");
                }
                return Ok(pool);
            }
            Ok(Err(err)) => {
                if !is_transient(&err) {
                    tracing::error!(error = %err, "Permanent connection error, not retrying");
                    return Err(err);
                }
                last_err = Some(err);
            }
            Err(_elapsed) => {
                // tokio::time::timeout fired — treat as transient I/O error
                last_err = Some(sqlx::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!(
                        "connection attempt timed out after {}s",
                        config.connect_timeout.as_secs()
                    ),
                )));
            }
        }

        if attempt < config.max_retries {
            let err_ref = last_err.as_ref().expect("just set");
            tracing::warn!(
                attempt,
                max_retries = config.max_retries,
                error = %err_ref,
                backoff_ms = millis(backoff),
                "Transient connection error, retrying"
            );
            tokio::time::sleep(backoff).await;
            backoff *= 2;
        }
    }

    // All attempts exhausted
    let err = last_err.expect("at least one attempt was made");
    tracing::error!(
        attempts = config.max_retries,
        elapsed_ms = millis(start.elapsed()),
        error = %err,
        "All connection attempts failed"
    );
    Err(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_matches_ticket_spec() {
        let cfg = PoolConfig::default();
        assert_eq!(cfg.connect_timeout, Duration::from_secs(15));
        assert_eq!(cfg.acquire_timeout, Duration::from_secs(10));
        assert_eq!(cfg.max_connections, 5);
        assert_eq!(cfg.min_connections, 0);
        assert_eq!(cfg.max_retries, 3);
        assert_eq!(cfg.initial_backoff, Duration::from_millis(500));
    }

    #[test]
    fn io_error_is_transient() {
        let err = sqlx::Error::Io(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "connection refused",
        ));
        assert!(is_transient(&err));
    }

    #[test]
    fn pool_timeout_is_transient() {
        let err = sqlx::Error::PoolTimedOut;
        assert!(is_transient(&err));
    }

    #[test]
    fn config_error_is_not_transient() {
        let err = sqlx::Error::Configuration("bad url".into());
        assert!(!is_transient(&err));
    }

    #[test]
    fn row_not_found_is_not_transient() {
        let err = sqlx::Error::RowNotFound;
        assert!(!is_transient(&err));
    }

    #[test]
    fn pool_closed_is_not_transient() {
        let err = sqlx::Error::PoolClosed;
        assert!(!is_transient(&err));
    }
}
