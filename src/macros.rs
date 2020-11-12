macro_rules! retry_impl {
    ($time:expr) => {
        use crate::{strategy::Immediate, RetryErr, RetryOp, RetryResult};

        use std::{future::Future, time::Duration};
        pub async fn retry_immediate<F, Fut, T, E>(f: F) -> Result<T, RetryErr<E>>
        where
            F: FnMut(RetryOp) -> Fut,
            Fut: Future<Output = RetryResult<T, E>>,
        {
            retry(Immediate, f).await
        }

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
    };
}
