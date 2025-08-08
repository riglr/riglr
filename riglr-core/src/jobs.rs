//! Job data structures and types

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A job represents a unit of work to be executed by a ToolWorker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique identifier for this job
    pub job_id: Uuid,
    /// Name of the tool to execute
    pub tool_name: String,
    /// Parameters to pass to the tool (JSON serialized)
    pub params: serde_json::Value,
    /// Optional idempotency key to prevent duplicate execution
    pub idempotency_key: Option<String>,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Current retry count
    #[serde(default)]
    pub retry_count: u32,
}

impl Job {
    /// Create a new job
    pub fn new<T: Serialize>(
        tool_name: impl Into<String>,
        params: &T,
        max_retries: u32,
    ) -> Result<Self, serde_json::Error> {
        Ok(Job {
            job_id: Uuid::new_v4(),
            tool_name: tool_name.into(),
            params: serde_json::to_value(params)?,
            idempotency_key: None,
            max_retries,
            retry_count: 0,
        })
    }

    /// Create a new job with an idempotency key
    pub fn new_idempotent<T: Serialize>(
        tool_name: impl Into<String>,
        params: &T,
        max_retries: u32,
        idempotency_key: impl Into<String>,
    ) -> Result<Self, serde_json::Error> {
        Ok(Job {
            job_id: Uuid::new_v4(),
            tool_name: tool_name.into(),
            params: serde_json::to_value(params)?,
            idempotency_key: Some(idempotency_key.into()),
            max_retries,
            retry_count: 0,
        })
    }

    /// Check if this job has retries remaining
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    /// Increment the retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

/// Result of executing a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobResult {
    /// Job executed successfully
    Success {
        /// Return value from the tool execution
        value: serde_json::Value,
        /// Optional transaction hash for on-chain operations
        tx_hash: Option<String>,
    },
    /// Job execution failed
    Failure {
        /// Error message describing the failure
        error: String,
        /// Whether this error is retriable
        retriable: bool,
    },
}

impl JobResult {
    /// Create a successful result
    pub fn success<T: Serialize>(value: &T) -> Result<Self, serde_json::Error> {
        Ok(JobResult::Success {
            value: serde_json::to_value(value)?,
            tx_hash: None,
        })
    }

    /// Create a successful result with transaction hash
    pub fn success_with_tx<T: Serialize>(
        value: &T,
        tx_hash: impl Into<String>,
    ) -> Result<Self, serde_json::Error> {
        Ok(JobResult::Success {
            value: serde_json::to_value(value)?,
            tx_hash: Some(tx_hash.into()),
        })
    }

    /// Create a retriable failure result
    pub fn retriable_failure(error: impl Into<String>) -> Self {
        JobResult::Failure {
            error: error.into(),
            retriable: true,
        }
    }

    /// Create a non-retriable failure result
    pub fn permanent_failure(error: impl Into<String>) -> Self {
        JobResult::Failure {
            error: error.into(),
            retriable: false,
        }
    }

    /// Check if this result represents a success
    pub fn is_success(&self) -> bool {
        matches!(self, JobResult::Success { .. })
    }

    /// Check if this result is a retriable failure
    pub fn is_retriable(&self) -> bool {
        matches!(self, JobResult::Failure { retriable: true, .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let params = serde_json::json!({"key": "value"});
        let job = Job::new("test_tool", &params, 3).unwrap();

        assert_eq!(job.tool_name, "test_tool");
        assert_eq!(job.params, params);
        assert_eq!(job.max_retries, 3);
        assert_eq!(job.retry_count, 0);
        assert!(job.idempotency_key.is_none());
        assert!(job.can_retry());
    }

    #[test]
    fn test_job_with_idempotency() {
        let params = serde_json::json!({"key": "value"});
        let job = Job::new_idempotent("test_tool", &params, 3, "test_key").unwrap();

        assert_eq!(job.idempotency_key, Some("test_key".to_string()));
    }

    #[test]
    fn test_job_retry_logic() {
        let params = serde_json::json!({"key": "value"});
        let mut job = Job::new("test_tool", &params, 2).unwrap();

        assert!(job.can_retry());
        job.increment_retry();
        assert!(job.can_retry());
        job.increment_retry();
        assert!(!job.can_retry());
    }

    #[test]
    fn test_job_result_creation() {
        let success = JobResult::success(&"test_value").unwrap();
        assert!(success.is_success());

        let success_with_tx = JobResult::success_with_tx(&"test_value", "tx_hash").unwrap();
        assert!(success_with_tx.is_success());

        let retriable_failure = JobResult::retriable_failure("test error");
        assert!(retriable_failure.is_retriable());
        assert!(!retriable_failure.is_success());

        let permanent_failure = JobResult::permanent_failure("test error");
        assert!(!permanent_failure.is_retriable());
        assert!(!permanent_failure.is_success());
    }
}