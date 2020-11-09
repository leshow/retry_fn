//! exponential backoff
use std::time::Duration;

#[derive(Debug, Copy, Clone)]
pub struct ExponentialBackoff {
    current: Duration,
    base: u32,
    max: Option<Duration>,
}

impl ExponentialBackoff {
    pub fn new(duration: Duration) -> Self {
        Self {
            current: duration,
            base: 2,
            max: None,
        }
    }

    pub fn base(mut self, base: u32) -> Self {
        self.base = base;
        self
    }

    pub fn max(mut self, max: Duration) -> Self {
        self.max = Some(max);
        self
    }

    pub fn from_millis(millis: u64) -> Self {
        Self::new(Duration::from_millis(millis))
    }

    pub fn from_secs(secs: u64) -> Self {
        Self::new(Duration::from_secs(secs))
    }

    pub fn from_micros(micros: u64) -> Self {
        Self::new(Duration::from_micros(micros))
    }

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
    fn returns_some_fixed() {
        let mut s = ExponentialBackoff::from_millis(100);
        assert_eq!(s.next(), Some(Duration::from_millis(200)));
        assert_eq!(s.next(), Some(Duration::from_millis(400)));
        assert_eq!(s.next(), Some(Duration::from_millis(800)));
    }

    #[test]
    fn returns_some_more() {
        let mut s = ExponentialBackoff::from_millis(100).base(10);
        assert_eq!(s.next(), Some(Duration::from_millis(1000)));
        assert_eq!(s.next(), Some(Duration::from_millis(10000)));
        assert_eq!(s.next(), Some(Duration::from_millis(100000)));
    }

    #[test]
    fn returns_max() {
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
