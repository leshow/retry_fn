//! # retry
//!
//! Function for executing retry either as a closure with a std-based sleep (`thread::sleep`) or
//! using either of the most popular async runtimes. Tokio or async-std
//!
//! ## Sync Example
//!
//! ```rust,no_run
//! # use std::{io, time::Duration};
//! use retry::{retry, RetryResult, strategy::ExponentialBackoff};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut count = 0;
//! let res = retry(ExponentialBackoff::new(Duration::from_secs(2)), |op| {
//!    if op.retries >= 3 {
//!        RetryResult::<&str, _>::Err(io::Error::new(
//!            io::ErrorKind::TimedOut,
//!            "timed out",
//!        ))
//!    } else {
//!        count += 1;
//!        RetryResult::Retry()
//!    }
//! });
//! assert_eq!(count, 3);
//! assert!(res.is_err());
//! Ok(())
//! # }
//! ```
//!
//! ## Using tokio
//! Enable the `tokio-runtime` feature to get access to this function
//!
//! ```rust,no_run
//! # use std::{io, sync::{Arc, Mutex}};
//! use retry::{tokio::retry, RetryResult, strategy::ConstantBackoff};
//! # use retry::tokio::retry;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # tokio::task::spawn_blocking(|| async move {
//! let count: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
//! let res = retry(ConstantBackoff::from_millis(100), |op| {
//!     let count = count.clone();
//!     async move {
//!         if op.retries >= 3 {
//!             RetryResult::<&str, _>::Err(io::Error::new(
//!                 io::ErrorKind::TimedOut,
//!                 "timed out",
//!             ))
//!         } else {
//!             *count.lock().unwrap() += 1;
//!             RetryResult::Retry()
//!         }
//!     }
//! })
//! .await;
//! assert_eq!(*count.lock().unwrap(), 3);
//! assert!(res.is_err());
//! # });
//! # Ok(())
//! # }
//! ```
#![warn(
    missing_debug_implementations,
    missing_docs,
    missing_copy_implementations,
    rust_2018_idioms,
    unreachable_pub,
    non_snake_case,
    non_upper_case_globals
)]
#![allow(clippy::cognitive_complexity)]
#![deny(broken_intra_doc_links)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

use crate::strategy::Immediate;

#[macro_use]
mod macros;
pub mod strategy;

#[cfg(feature = "tokio-runtime")]
pub mod tokio;

#[cfg(feature = "async-runtime")]
pub mod async_std;

use std::{error::Error, fmt, thread, time::Duration};

/// `RetryOp` gives some inspection into the current state of retries
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RetryOp {
    /// number of retries
    pub retries: usize,
    /// total duration we've delayed
    pub total_delay: Duration,
}

/// What to do with the current result of the function
///
/// `Retry` will execute the function again, `Err(E)` will return an error with E,
/// `Ok(T)` will return success with T
#[derive(Debug, Clone)]
pub enum RetryResult<T, E> {
    /// try again
    Retry(),
    /// return with an error
    Err(E),
    /// return with success
    Ok(T),
}

/// Error type for retry
#[derive(Debug, Clone)]
pub enum RetryErr<E> {
    /// Attempt failed with an error
    FailedAttempt {
        /// number of attempts
        tries: usize,
        /// total delay
        total_delay: Duration,
        /// the error
        err: E,
    },
    /// Attempt failed by reaching the end of the iterator
    IteratorEnded {
        /// number of attempts
        tries: usize,
        /// total delay
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

/// retry with the 'immediate' strategy, i.e. no wait in between attempts
///
/// ```rust,no_run
/// # use std::io;
/// use retry::{retry_immediate, RetryResult};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut count = 0;
/// let res = retry_immediate(|op| {
///    if op.retries >= 3 {
///        RetryResult::<&str, _>::Err(io::Error::new(
///            io::ErrorKind::TimedOut,
///            "timed out",
///        ))
///    } else {
///        count += 1;
///        RetryResult::Retry()
///    }
/// });
/// assert_eq!(count, 3);
/// assert!(res.is_err());
/// Ok(())
/// # }
/// ```
///
/// # Returns
/// If successful, return `Ok`, otherwise return `Retry` to try again or `Err` to exit
/// with an error
pub fn retry_immediate<F, T, E>(f: F) -> Result<T, RetryErr<E>>
where
    F: FnMut(RetryOp) -> RetryResult<T, E>,
{
    retry(Immediate, f)
}

/// Retry a function on some time interval
///
/// ```rust,no_run
/// # use std::{io, time::Duration};
/// use retry::{retry, RetryResult, strategy::ExponentialBackoff};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut count = 0;
/// let res = retry(ExponentialBackoff::new(Duration::from_secs(2)), |op| {
///    if op.retries >= 3 {
///        RetryResult::<&str, _>::Err(io::Error::new(
///            io::ErrorKind::TimedOut,
///            "timed out",
///        ))
///    } else {
///        count += 1;
///        RetryResult::Retry()
///    }
/// });
/// assert_eq!(count, 3);
/// assert!(res.is_err());
/// Ok(())
/// # }
/// ```
///
/// # Returns
/// If successful, return `Ok`, otherwise return `Retry` to try again or `Err` to exit
/// with an error
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
        let res = retry(ConstantBackoff::from_millis(100), |op| {
            if op.retries >= 3 {
                RetryResult::<usize, &str>::Ok(5)
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
