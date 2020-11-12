use crate::strategy::Immediate;

#[macro_use]
mod macros;
mod strategy;

#[cfg(feature = "tokio-runtime")]
pub mod tokio;

#[cfg(feature = "async-runtime")]
pub mod async_std;

use std::{error::Error, fmt, thread, time::Duration};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RetryOp {
    retries: usize,
    total_delay: Duration,
}

#[derive(Debug, Clone)]
pub enum RetryResult<T, E> {
    Retry(),
    Err(E),
    Ok(T),
}

#[derive(Debug, Clone)]
pub enum RetryErr<E> {
    FailedAttempt {
        tries: usize,
        total_delay: Duration,
        err: E,
    },
    IteratorEnded {
        tries: usize,
        total_delay: Duration,
    },
}

impl<E> Error for RetryErr<E> where E: fmt::Display + fmt::Debug {}

impl<E> fmt::Display for RetryErr<E>
where
    E: fmt::Display + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RetryErr::FailedAttempt {
                tries,
                total_delay,
                err,
            } => write!(
                f,
                "failed with {}, tries {} total delay {:#?}",
                err, tries, total_delay
            ),
            RetryErr::IteratorEnded { tries, total_delay } => write!(
                f,
                "iterator ended, retries {}, total delay {:#?}",
                tries, total_delay
            ),
        }
    }
}

pub fn retry_immediate<F, T, E>(f: F) -> Result<T, RetryErr<E>>
where
    F: FnMut(RetryOp) -> RetryResult<T, E>,
{
    retry(Immediate, f)
}

pub fn retry<I, F, T, E>(iter: I, mut f: F) -> Result<T, RetryErr<E>>
where
    I: IntoIterator<Item = Duration>,
    F: FnMut(RetryOp) -> RetryResult<T, E>,
{
    let mut count = 0;
    let mut total_delay = Duration::from_millis(0);
    for dur in iter.into_iter() {
        match f(RetryOp {
            retries: count,
            total_delay,
        }) {
            RetryResult::Retry() => {
                thread::sleep(dur);
                total_delay += dur;
                count += 1;
            }
            RetryResult::Err(err) => {
                return Err(RetryErr::FailedAttempt {
                    tries: count,
                    total_delay,
                    err,
                });
            }
            RetryResult::Ok(val) => {
                return Ok(val);
            }
        }
    }
    Err(RetryErr::IteratorEnded {
        tries: count,
        total_delay,
    })
}

#[cfg(test)]
mod test {
    use crate::RetryResult;

    use super::*;
    use crate::strategy::*;

    use std::io;

    #[test]
    fn fail_on_three() -> io::Result<()> {
        let mut count = 0;
        let res = retry(ConstantBackoff::from_millis(100), |op| {
            if op.retries >= 3 {
                RetryResult::<&str, _>::Err(io::Error::new(io::ErrorKind::TimedOut, "timed out"))
            } else {
                count += 1;
                RetryResult::Retry()
            }
        });
        assert_eq!(count, 3);
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    fn pass_eventually() -> io::Result<()> {
        let mut count = 0;
        let res = retry::<_, _, _, &str>(ConstantBackoff::from_millis(100), |op| {
            if op.retries >= 3 {
                RetryResult::Ok(5)
            } else {
                count += 1;
                RetryResult::Retry()
            }
        });
        assert_eq!(count, 3);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 5);

        Ok(())
    }
}
