mod strategy;

use futures::stream;
use std::future::Future;
use std::io;
use std::time::Duration;
use tokio::time;

pub enum RetryResult<T, E> {
    Retry(E),
    Err(E),
    Ok(T),
}

pub enum RetryErr<E> {
    FailedTry {
        tries: usize,
        total_delay: Duration,
        err: E,
    },
    IteratorEnded {
        tries: usize,
        total_delay: Duration,
    },
}

pub struct RetryOp<T> {
    retries: usize,
    value: T,
}

async fn retry<I, F, Fut, T, E>(iter: I, mut f: F) -> Result<T, RetryErr<E>>
where
    I: IntoIterator<Item = Duration>,
    F: FnMut() -> Fut,
    Fut: Future<Output = RetryResult<T, E>>,
{
    let mut count = 0;
    let mut total_delay = Duration::from_millis(0);
    for dur in iter.into_iter() {
        match f().await {
            RetryResult::Retry(err) => {
                time::delay_for(dur).await;
                total_delay += dur;
                count += 1;
            }
            RetryResult::Err(err) => {
                return Err(RetryErr::FailedTry {
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
    use super::*;

    #[tokio::test]
    async fn test() {}
}
/*
retry(ContanstantBackoff::new(), |res| async move {
    match res {
        Ok(d) if d.retries => {
            // do something
        }

    }
})
*/
