//! exponential backoff
//!
//! Exponential time backoff, starting at some time, increase by multiplying by
//! some base ex. base = 2, start = 2ms |--|----|--------|----------------|
//! Up to some optional maximum duration
//! ```rust
//! # use retry_fn::strategy::ExponentialBackoff;
//! # use std::time::Duration;
//!
//! let mut s = ExponentialBackoff::from_millis(100);
//! assert_eq!(s.next(), Some(Duration::from_millis(200)));
//! assert_eq!(s.next(), Some(Duration::from_millis(400)));
//! assert_eq!(s.next(), Some(Duration::from_millis(800)));
//! ```
use std::time::Duration;

/// Define a type for the exponential time iterator
#[derive(Debug, Copy, Clone)]
pub struct ExponentialBackoff {
    current: Duration,
    base: u32,
    max: Option<Duration>,
}

impl ExponentialBackoff {
    /// Create a new exp type with a starting duration
    pub fn new(first: Duration) -> Self {
        Self {
            current: first,
            base: 2,
            max: None,
        }
    }

    /// Set the base that we will multiply the series with
    /// base 2 is the default
    pub fn base(mut self, base: u32) -> Self {
        self.base = base;
        self
    }

    /// The maximum time the series will allow
    pub fn max(mut self, max: Duration) -> Self {
        self.max = Some(max);
        self
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

impl Iterator for ExponentialBackoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self
            .current
            .checked_mul(self.base)
            .unwrap_or_else(|| Duration::from_millis(u64::MAX));
        self.current = next;

        match self.max {
            Some(m) if m <= next => self.max,
            _ => Some(next),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn doubles() {
        let mut s = ExponentialBackoff::from_millis(100);
        assert_eq!(s.next(), Some(Duration::from_millis(200)));
        assert_eq!(s.next(), Some(Duration::from_millis(400)));
        assert_eq!(s.next(), Some(Duration::from_millis(800)));
    }

    #[test]
    fn tens() {
        let mut s = ExponentialBackoff::from_millis(100).base(10);
        assert_eq!(s.next(), Some(Duration::from_millis(1000)));
        assert_eq!(s.next(), Some(Duration::from_millis(10000)));
        assert_eq!(s.next(), Some(Duration::from_millis(100000)));
    }

    #[test]
    fn hits_max() {
        let mut s = ExponentialBackoff::from_millis(100)
            .base(10)
            .max(Duration::from_millis(1_000_000));
        assert_eq!(s.next(), Some(Duration::from_millis(1_000)));
        assert_eq!(s.next(), Some(Duration::from_millis(10_000)));
        assert_eq!(s.next(), Some(Duration::from_millis(100_000)));
        assert_eq!(s.next(), Some(Duration::from_millis(1_000_000)));
        assert_eq!(s.next(), Some(Duration::from_millis(1_000_000)));
        assert_eq!(s.next(), Some(Duration::from_millis(1_000_000)));
    }
}
