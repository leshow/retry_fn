retry_impl!(async_std::task::sleep);

#[cfg(test)]
mod test {
    use crate::RetryResult;

    use super::*;
    use crate::strategy::*;
    use std::time::Duration;

    use std::{
        io,
        sync::{Arc, Mutex},
    };
}
