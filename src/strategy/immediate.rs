//! immediate return
//!
//! This distribution just returns immediately, using 0 as it's Duration
use std::time::Duration;

/// Define type for Immediate strategy
#[derive(Debug, Default, Copy, Clone)]
pub struct Immediate;

impl Immediate {
    /// Create new `Immediate`
    /// (not necessary, you can just use `Immediate` since it holds no data)
    pub fn new() -> Self {
        Self
    }
}

impl Iterator for Immediate {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Duration::from_millis(0))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn fixed() {
        let mut s = Immediate::new();
        assert_eq!(s.next(), Some(Duration::from_millis(0)));
        assert_eq!(s.next(), Some(Duration::from_millis(0)));
        assert_eq!(s.next(), Some(Duration::from_millis(0)));
    }
}
