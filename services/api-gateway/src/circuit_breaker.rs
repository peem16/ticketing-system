//! Simple circuit breaker for protecting downstream gRPC calls
//!
//! States:
//! - **Closed**: requests pass through normally
//! - **Open**: requests are immediately rejected after `threshold` consecutive failures
//! - **Half-Open**: after `recovery_timeout`, one probe request is allowed through

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Thread-safe circuit breaker.
#[derive(Clone)]
pub struct CircuitBreaker {
    inner: Arc<Inner>,
}

struct Inner {
    failure_count: AtomicU32,
    is_open: AtomicBool,
    /// Milliseconds since UNIX epoch of the last recorded failure
    last_failure_ms: AtomicU64,
    threshold: u32,
    recovery_timeout: Duration,
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    ///
    /// # Arguments
    /// * `threshold` - Number of consecutive failures before the circuit opens
    /// * `recovery_timeout` - How long to wait before allowing a probe request
    pub fn new(threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            inner: Arc::new(Inner {
                failure_count: AtomicU32::new(0),
                is_open: AtomicBool::new(false),
                last_failure_ms: AtomicU64::new(0),
                threshold,
                recovery_timeout,
            }),
        }
    }

    /// Check whether a request should be allowed through.
    ///
    /// Returns `true` if the circuit is closed (or half-open and ready to probe).
    pub fn is_available(&self) -> bool {
        if !self.inner.is_open.load(Ordering::Acquire) {
            return true;
        }

        // Circuit is open — check if recovery timeout has elapsed
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let last_ms = self.inner.last_failure_ms.load(Ordering::Acquire);
        let elapsed = Duration::from_millis(now_ms.saturating_sub(last_ms));

        elapsed >= self.inner.recovery_timeout
    }

    /// Record a successful response — resets the failure counter and closes the circuit.
    pub fn record_success(&self) {
        self.inner.failure_count.store(0, Ordering::Release);
        self.inner.is_open.store(false, Ordering::Release);
    }

    /// Record a failed response — increments the failure counter and potentially opens the circuit.
    pub fn record_failure(&self) {
        let count = self.inner.failure_count.fetch_add(1, Ordering::AcqRel) + 1;
        if count >= self.inner.threshold {
            self.inner.is_open.store(true, Ordering::Release);
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            self.inner.last_failure_ms.store(now_ms, Ordering::Release);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starts_closed() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(5));
        assert!(cb.is_available());
    }

    #[test]
    fn test_opens_after_threshold() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        cb.record_failure();
        cb.record_failure();
        assert!(cb.is_available()); // still below threshold
        cb.record_failure();
        assert!(!cb.is_available()); // now open
    }

    #[test]
    fn test_resets_on_success() {
        let cb = CircuitBreaker::new(2, Duration::from_secs(60));
        cb.record_failure();
        cb.record_failure();
        assert!(!cb.is_available());
        cb.record_success();
        assert!(cb.is_available());
    }

    #[test]
    fn test_half_open_after_recovery_timeout() {
        let cb = CircuitBreaker::new(1, Duration::from_millis(0));
        cb.record_failure();
        // recovery_timeout = 0ms, so it should be available immediately
        assert!(cb.is_available());
    }
}
