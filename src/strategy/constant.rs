//! constant time backoff
//!
//! Just sets a constant amount of time between each retry
//! ex. |---|---|---|---|
//!
//! ```rust
//! # use retry::strategy::ConstantBackoff;
//! # use std::time::Duration;
//! let mut s = ConstantBackoff::new(Duration::from_millis(100));
//! assert_eq!(s.next(), Some(Duration::from_millis(100)));
//! assert_eq!(s.next(), Some(Duration::from_millis(100)));
//! assert_eq!(s.next(), Some(Duration::from_millis(100)));
//! ```
use std::time::Duration;

/// Create a new type representing a constant time iterator
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ConstantBackoff {
    duration: Duration,
}

impl From<Duration> for ConstantBackoff {
    fn from(duration: Duration) -> Self {
        Self { duration }
    }
}

impl ConstantBackoff {
    /// Create a new `ConstantBackoff`
    pub fn new(duration: Duration) -> Self {
        duration.into()
    }

    /// create a new type using n milliseconds as the start value
    pub fn from_millis(millis: u64) -> Self {
        Self::new(Duration::from_millis(millis))
    }

    /// create a new type using n seconds as the start value
    pub fn from_secs(secs: u64) -> Self {
        Self::new(Duration::from_secs(secs))
    }

    /// create a new type using n microseconds as the start value
    pub fn from_micros(micros: u64) -> Self {
        Self::new(Duration::from_micros(micros))
    }

    /// create a new type using n nanoseconds as the start value
    pub fn from_nanos(nanos: u64) -> Self {
        Self::new(Duration::from_nanos(nanos))
    }
}

impl Iterator for ConstantBackoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.duration)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn fixed() {
        let mut s = ConstantBackoff::new(Duration::from_millis(100));
        assert_eq!(s.next(), Some(Duration::from_millis(100)));
        assert_eq!(s.next(), Some(Duration::from_millis(100)));
        assert_eq!(s.next(), Some(Duration::from_millis(100)));
    }

    #[test]
    fn fixed_secs() {
        let mut s = ConstantBackoff::new(Duration::from_secs(1));
        assert_eq!(s.next(), Some(Duration::from_secs(1)));
        assert_eq!(s.next(), Some(Duration::from_secs(1)));
        assert_eq!(s.next(), Some(Duration::from_secs(1)));
    }
}
