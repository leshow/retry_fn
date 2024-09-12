//! retry impls for tokio
//!
//! Enable the `tokio-runtime` feature to get access to this function
//!
//! ```rust,no_run
//! # use std::{io, sync::{Arc, Mutex}};
//! use retry_fn::{tokio::retry, strategy::Constant, RetryResult};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # tokio::task::spawn_blocking(|| async move {
//! let count: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
//! let res = retry(Constant::from_millis(100), |op| {
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

retry_impl!(tokio::time::sleep);

#[cfg(test)]
mod test {
    use crate::RetryResult;

    use super::*;
    use crate::strategy::*;

    use std::{
        io,
        sync::{Arc, Mutex},
    };

    #[tokio::test]
    async fn fail_on_three() -> io::Result<()> {
        let count: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
        let res = retry(Constant::from_millis(100), |op| {
            let count = count.clone();
            async move {
                if op.retries >= 3 {
                    RetryResult::<&str, _>::Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "timed out",
                    ))
                } else {
                    *count.lock().unwrap() += 1;
                    RetryResult::Retry()
                }
            }
        })
        .await;
        assert_eq!(*count.lock().unwrap(), 3);
        assert!(res.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn pass_eventually() -> io::Result<()> {
        let count = Arc::new(Mutex::new(0));
        let res = retry(Constant::from_millis(100), |op| {
            let count = count.clone();
            async move {
                if op.retries >= 3 {
                    RetryResult::<&str, &str>::Err("failed on 3")
                } else {
                    *count.lock().unwrap() += 1;
                    RetryResult::Retry()
                }
            }
        })
        .await;
        assert_eq!(*count.lock().unwrap(), 3);
        assert!(res.is_err());

        Ok(())
    }
}
