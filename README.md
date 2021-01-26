# retry_fn

[![Build Status](https://github.com/leshow/retry_fn/workflows/Actions/badge.svg)](https://github.com/leshow/retry_fn/actions)
[![Crate](https://img.shields.io/crates/v/retry_fn.svg)](https://crates.io/crates/retry_fn)
[![API](https://docs.rs/retry_fn/badge.svg)](https://docs.rs/retry_fn)

Function for executing retry either as a closure with a std-based sleep (`thread::sleep`) or
using either of the most popular async runtime's. Tokio or async-std. Inspired by the other
retry libraries out there, the desire is to keep this up-to-date and combine features from several.

## Sync Example

```rust
use std::{io, time::Duration};
use retry_fn::{retry, RetryResult, strategy::ExponentialBackoff};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut count = 0;
    let res = retry(ExponentialBackoff::new(Duration::from_secs(2)), |op| {
       if op.retries >= 3 {
           RetryResult::<&str, _>::Err(io::Error::new(
               io::ErrorKind::TimedOut,
               "timed out",
           ))
       } else {
           count += 1;
           RetryResult::Retry()
       }
    });
    assert_eq!(count, 3);
    assert!(res.is_err());
    Ok(())
}
```

## Using tokio

Enable the `tokio-runtime` feature to get access to this function

```rust
use std::{io, sync::{Arc, Mutex}};
use retry_fn::{tokio::retry, RetryResult, strategy::Constant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
```
