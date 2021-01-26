macro_rules! retry_impl {
    ($time:expr) => {
        use crate::{RetryErr, RetryOp, RetryResult};
        use std::{future::Future, time::Duration};

        /// Retry a future based on an iterator over Duration. A timer will be run for
        /// each item in the iterator.
        ///
        /// ```rust,no_run
        /// # use std::{io, sync::{Arc, Mutex}};
        /// use retry_fn::strategy::Constant;
        /// use retry_fn::RetryResult;
        /// # use retry_fn::tokio::retry;
        /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
        /// # tokio::task::spawn_blocking(|| async move {
        /// let count: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
        /// let res = retry(Constant::from_millis(100), |op| {
        ///     let count = count.clone();
        ///     async move {
        ///         if op.retries >= 3 {
        ///             RetryResult::<&str, _>::Err(io::Error::new(
        ///                 io::ErrorKind::TimedOut,
        ///                 "timed out",
        ///             ))
        ///         } else {
        ///             *count.lock().unwrap() += 1;
        ///             RetryResult::Retry()
        ///         }
        ///     }
        /// })
        /// .await;
        /// assert_eq!(*count.lock().unwrap(), 3);
        /// assert!(res.is_err());
        /// # });
        /// # Ok(())
        /// # }
        /// ```
        ///
        /// # Returns
        /// If successful, return `Ok`, otherwise return `Retry` to try again or `Err`
        /// to exit with an error
        pub async fn retry<I, F, Fut, T, E>(iter: I, mut f: F) -> Result<T, RetryErr<E>>
        where
            I: IntoIterator<Item = Duration>,
            F: FnMut(RetryOp) -> Fut,
            Fut: Future<Output = RetryResult<T, E>>,
        {
            let mut count = 0;
            let mut total_delay = Duration::from_millis(0);
            for dur in iter.into_iter() {
                match f(RetryOp {
                    retries: count,
                    total_delay,
                })
                .await
                {
                    RetryResult::Retry() => {
                        $time(dur).await;
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

        /// Retry a future based on an iterator over Duration. A timer will be run for
        /// each item in the iterator.
        /// This takes a future that must implement `Unpin`, so it can be repeatedly
        /// called in the loop
        ///
        /// ```rust,no_run
        /// # use std::{io, sync::{Arc, Mutex}};
        /// use retry_fn::strategy::Constant;
        /// use retry_fn::RetryResult;
        /// # use retry_fn::tokio::retry_unpin;
        /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
        /// # tokio::task::spawn_blocking(|| async move {
        /// let count: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
        /// let fut = {
        ///     let count = count.clone();
        ///     async move {
        ///         if *count.lock().unwrap() >= 3 {
        ///             RetryResult::<&str, _>::Err(io::Error::new(
        ///                 io::ErrorKind::TimedOut,
        ///                 "timed out",
        ///             ))
        ///         } else {
        ///             *count.lock().unwrap() += 1;
        ///             RetryResult::Retry()
        ///         }
        ///     }
        /// };
        /// tokio::pin!(fut);
        /// let res = retry_unpin(Constant::from_millis(100), fut)
        /// .await;
        /// assert_eq!(*count.lock().unwrap(), 3);
        /// assert!(res.is_err());
        /// # });
        /// # Ok(())
        /// # }
        /// ```
        ///
        /// # Returns
        /// If successful, return `Ok`, otherwise return `Retry` to try again or `Err`
        /// to exit with an error
        pub async fn retry_unpin<I, Fut, T, E>(iter: I, mut f: Fut) -> Result<T, RetryErr<E>>
        where
            I: IntoIterator<Item = Duration>,
            Fut: Future<Output = RetryResult<T, E>> + Unpin,
        {
            let mut count = 0;
            let mut total_delay = Duration::from_millis(0);
            for dur in iter.into_iter() {
                match (&mut f).await {
                    RetryResult::Retry() => {
                        tokio::time::sleep(dur).await;
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
    };
}
