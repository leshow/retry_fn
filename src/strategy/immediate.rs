//! constant time backoff
//!
use std::time::Duration;

#[derive(Debug)]
pub struct Immediate;

impl Immediate {
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
