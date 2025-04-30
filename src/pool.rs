/// our own pool so we can dynamically adjust the capacity
use std::vec::Vec;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
// use std::time::{Duration, Instant};

pub struct Pool<T, F>
where
    F: Future<Output = T>,
{
    stack: Vec<T>,
    pub capacity: usize,
    // TODO max_age? / max_idle_s?
    create: Box<dyn FnMut() -> F>,
    waker: Option<Waker>,
}

impl<T, F: Future<Output=T>> Pool<T, F> {
    pub fn new<C: FnMut() -> F + 'static>(capacity: usize, create: C) -> Self {
        Pool {
            stack: Vec::new(),
            capacity,
            create: Box::new(create),
            waker: None,
        }
    }

    // pub fn take() -> PoolFuture<T>()
}
