//! retry impls for async-std

retry_impl!(async_std::task::sleep);

#[cfg(test)]
mod test {
    use crate::RetryResult;

    use super::*;
    use crate::strategy::*;

    use async_std::task;
    use std::{
        io,
        sync::{Arc, Mutex},
    };

    #[test]
    fn fail_on_three() {
        task::block_on(async {
            let count: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
            let res = retry(ConstantBackoff::from_millis(100), |op| {
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
        });
    }

    #[test]
    fn pass_eventually() {
        task::block_on(async {
            let count = Arc::new(Mutex::new(0));
            let res = retry(ConstantBackoff::from_millis(100), |op| {
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
        });
    }
}
