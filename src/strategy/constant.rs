use std::time::Duration;

#[derive(Debug)]
pub struct ConstantBackoff {
    duration: Duration,
}

impl From<Duration> for ConstantBackoff {
    fn from(duration: Duration) -> Self {
        Self { duration }
    }
}

impl ConstantBackoff {
    pub fn new(duration: Duration) -> Self {
        duration.into()
    }

    pub fn from_millis(millis: u64) -> Self {
        Self {
            duration: Duration::from_millis(millis),
        }
    }

    pub fn from_secs(secs: u64) -> Self {
        Self {
            duration: Duration::from_secs(secs),
        }
    }

    pub fn from_micros(micros: u64) -> Self {
        Self {
            duration: Duration::from_micros(micros),
        }
    }

    pub fn from_nanos(nanos: u64) -> Self {
        Self {
            duration: Duration::from_nanos(nanos),
        }
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
    fn returns_some_fixed() {
        let mut s = ConstantBackoff::new(Duration::from_millis(100));
        assert_eq!(s.next(), Some(Duration::from_millis(100)));
        assert_eq!(s.next(), Some(Duration::from_millis(100)));
        assert_eq!(s.next(), Some(Duration::from_millis(100)));
    }
}
