/// our own queue so we can dynamically adjust the capacity, and push_front even when at capacity
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

pub fn queue<T: Sync>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let q = Queue::new(capacity);
    let arc = Arc::new(Mutex::new(q));
    (Sender(arc.clone()), Receiver(arc))
}

struct Queue<T: Sync> {
    queue: VecDeque<Metadata<T>>,
    capacity: usize,
    waker: Option<Waker>,
}

impl<T: Sync> Queue<T> {
    pub fn new(capacity: usize) -> Self {
        Queue {
            queue: VecDeque::new(),
            capacity,
            waker: None,
        }
    }

    pub fn free_space(&self) -> usize {
        self.capacity - self.queue.len()
    }
}

pub struct Sender<T: Sync>(Arc<Mutex<Queue<T>>>);

impl<T: Sync> Sender<T> {
    /// Caller must check whether provided vector is drained or has unused values
    pub fn send_many(&self, ttl: Duration, values: &mut Vec<T>) -> usize {
        let arrival = Instant::now();
        let deadline = arrival + ttl;
        let mut q = self.0.lock().unwrap(); // not safe, should crash process
        let consumed = usize::min(q.free_space(), values.len());
        if consumed > 0 {
            if let Some(waker) = q.waker.take() {
                waker.wake();
            }
        }
        for _ in 0..consumed {
            let value = values.pop().unwrap(); // safe because we pop at most values.len times
            q.queue.push_back(Metadata {
                value,
                arrival,
                deadline,
            });
        }
        consumed
    }
}

pub struct Receiver<T: Sync>(Arc<Mutex<Queue<T>>>);

impl<T: Sync> Receiver<T> {
    pub fn recv(&self) -> QueueFuture<T> {
        QueueFuture(self.0.clone())
    }

    pub fn set_capacity(&self, capacity: usize) {
        let mut q = self.0.lock().unwrap(); // not safe, should crash process
        q.capacity = capacity;
    }

    // ignores capacity
    pub fn push_front(&self, value: Metadata<T>) {
        let mut q = self.0.lock().unwrap(); // not safe, should crash process
        q.queue.push_front(value);
    }
}

pub struct Metadata<T> {
    value: T,
    arrival: Instant,
    deadline: Instant,
}

pub struct QueueFuture<T: Sync>(Arc<Mutex<Queue<T>>>);

impl<T: Sync> Future for QueueFuture<T> {
    type Output = Metadata<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut q = self.0.lock().unwrap(); // not safe, should crash process
        match q.queue.pop_front() {
            Some(meta) => Poll::Ready(meta),
            None => {
                let waker = cx.waker();
                q.waker = Some(waker.clone());
                Poll::Pending
            }
        }
    }
}
