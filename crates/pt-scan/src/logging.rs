//! Structured classification logging for ML training data collection.
//!
//! Every BAML/LLM classification call can be logged as a JSONL record to
//! `data/classification_log/`. Each record is a potential training example
//! for distilling a lightweight classifier.
//!
//! Logging is opt-in: set `PLANTASTIC_LOG_CLASSIFICATIONS=1` to enable.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;

use crate::feature::FeatureCandidate;
use crate::pipeline::ClassifiedFeatureOutput;

const LOG_DIR: &str = "data/classification_log";
const ENV_VAR: &str = "PLANTASTIC_LOG_CLASSIFICATIONS";

/// Context metadata for a classification run.
#[derive(Debug, Clone, Serialize)]
pub struct ClassificationContext {
    pub address: String,
    pub climate_zone: String,
}

/// A single classification log record (one per pipeline run).
#[derive(Debug, Serialize)]
pub struct ClassificationRecord {
    pub timestamp: String,
    pub scan_id: String,
    pub candidates: Vec<FeatureCandidate>,
    pub classifications: Vec<ClassifiedFeatureOutput>,
    pub context: ClassificationContext,
}

/// JSONL logger for classification training data.
///
/// Writes append-only JSONL files to `data/classification_log/`.
/// Each line is a self-contained JSON object suitable for:
/// `cat data/classification_log/*.jsonl | python train.py`
#[derive(Debug)]
pub struct ClassificationLogger {
    scan_id: String,
    enabled: bool,
    log_dir: PathBuf,
}

impl ClassificationLogger {
    /// Create a logger for the given scan.
    ///
    /// Checks the `PLANTASTIC_LOG_CLASSIFICATIONS` env var. If unset or "0",
    /// all `log()` calls are no-ops.
    pub fn new(scan_id: impl Into<String>) -> Self {
        let enabled = std::env::var(ENV_VAR)
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        Self {
            scan_id: scan_id.into(),
            enabled,
            log_dir: PathBuf::from(LOG_DIR),
        }
    }

    /// Create a logger with a custom output directory (for testing).
    #[cfg(test)]
    fn with_dir(scan_id: impl Into<String>, log_dir: PathBuf) -> Self {
        Self {
            scan_id: scan_id.into(),
            enabled: true,
            log_dir,
        }
    }

    /// Whether logging is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Log a classification result.
    ///
    /// No-op if logging is disabled. Creates the log directory if needed.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if directory creation or file writing fails.
    pub fn log(
        &self,
        candidates: &[FeatureCandidate],
        classifications: &[ClassifiedFeatureOutput],
        context: &ClassificationContext,
    ) -> Result<(), std::io::Error> {
        if !self.enabled {
            return Ok(());
        }

        fs::create_dir_all(&self.log_dir)?;

        let timestamp = now_iso8601();
        let record = ClassificationRecord {
            timestamp: timestamp.clone(),
            scan_id: self.scan_id.clone(),
            candidates: candidates.to_vec(),
            classifications: classifications.to_vec(),
            context: context.clone(),
        };

        let json = serde_json::to_string(&record)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let filename = format!("{}_{}.jsonl", self.scan_id, sanitize_timestamp(&timestamp));
        let path = self.log_dir.join(filename);

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        writeln!(file, "{json}")?;
        Ok(())
    }
}

/// ISO 8601 timestamp string.
fn now_iso8601() -> String {
    // Use std::time for a simple UTC timestamp without pulling in chrono.
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Simple UTC formatting: YYYY-MM-DDTHH:MM:SSZ
    // Epoch arithmetic for year/month/day
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Days since 1970-01-01 to Y-M-D (simplified leap year handling)
    let (year, month, day) = days_to_ymd(days);

    format!("{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

/// Convert days since epoch to (year, month, day).
fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let months: [u64; 12] = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for &m in &months {
        if days < m {
            break;
        }
        days -= m;
        month += 1;
    }

    (year, month, days + 1)
}

fn is_leap(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Replace characters that are problematic in filenames.
fn sanitize_timestamp(ts: &str) -> String {
    ts.replace(':', "-").replace('T', "_").replace('Z', "")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pt_test_utils::timed;

    fn sample_candidate() -> FeatureCandidate {
        FeatureCandidate {
            cluster_id: 0,
            centroid: [1.0, 2.0, 3.0],
            bbox_min: [0.5, 1.5, 2.5],
            bbox_max: [1.5, 2.5, 3.5],
            height_ft: 10.0,
            spread_ft: 5.0,
            point_count: 100,
            dominant_color: "green".to_string(),
            vertical_profile: "spreading".to_string(),
            density: 50.0,
        }
    }

    fn sample_classification() -> ClassifiedFeatureOutput {
        ClassifiedFeatureOutput {
            cluster_id: 0,
            label: "London Plane".to_string(),
            category: "tree".to_string(),
            species: Some("Platanus × acerifolia".to_string()),
            confidence: 0.85,
            reasoning: "tall spreading tree".to_string(),
            landscape_notes: "provides shade".to_string(),
        }
    }

    #[test]
    fn logger_disabled_by_default() {
        timed(|| {
            // Ensure env var is not set for this test
            std::env::remove_var(ENV_VAR);
            let logger = ClassificationLogger::new("test-scan");
            assert!(!logger.is_enabled());
        });
    }

    #[test]
    fn logger_writes_valid_jsonl() {
        timed(|| {
            let dir = std::env::temp_dir().join(format!("pt-scan-log-test-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);

            let logger = ClassificationLogger::with_dir("test-scan", dir.clone());
            let ctx = ClassificationContext {
                address: "Powell & Market, SF".to_string(),
                climate_zone: "USDA 10b".to_string(),
            };

            logger
                .log(&[sample_candidate()], &[sample_classification()], &ctx)
                .expect("log should succeed");

            // Find the written file
            let entries: Vec<_> = fs::read_dir(&dir)
                .expect("log dir should exist")
                .filter_map(Result::ok)
                .collect();
            assert_eq!(entries.len(), 1, "expected exactly 1 log file");

            let content = fs::read_to_string(entries[0].path()).expect("should read log");
            let lines: Vec<&str> = content.lines().collect();
            assert_eq!(lines.len(), 1, "expected exactly 1 JSONL line");

            // Validate it's valid JSON
            let record: serde_json::Value =
                serde_json::from_str(lines[0]).expect("should be valid JSON");
            assert_eq!(record["scan_id"], "test-scan");
            assert_eq!(record["candidates"][0]["cluster_id"], 0);
            assert_eq!(record["classifications"][0]["label"], "London Plane");
            assert_eq!(record["context"]["address"], "Powell & Market, SF");

            // Cleanup
            let _ = fs::remove_dir_all(&dir);
        });
    }

    #[test]
    fn logger_noop_when_disabled() {
        timed(|| {
            std::env::remove_var(ENV_VAR);
            let logger = ClassificationLogger::new("noop-scan");

            let ctx = ClassificationContext {
                address: "test".to_string(),
                climate_zone: "test".to_string(),
            };

            // Should succeed without writing anything
            logger
                .log(&[sample_candidate()], &[sample_classification()], &ctx)
                .expect("noop log should succeed");
        });
    }
}
