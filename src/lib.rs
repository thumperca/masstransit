use std::collections::VecDeque;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread::Thread;

struct WaitingThread {
    thread: Thread,
    count: usize,
}

impl WaitingThread {
    pub fn new(count: usize) -> Self {
        Self {
            thread: std::thread::current(),
            count,
        }
    }
}

struct ChannelInner<T> {
    queue: Mutex<VecDeque<WaitingThread>>,
    data: Mutex<VecDeque<T>>,
}

impl<T> ChannelInner<T> {
    fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            data: Mutex::new(VecDeque::new()),
        }
    }
}

#[derive(Clone)]
struct Channel<T> {
    inner: Arc<ChannelInner<T>>,
}

impl<T> Channel<T> {
    fn new() -> Self {
        Self {
            inner: Arc::new(ChannelInner::new()),
        }
    }
}

impl<T> Deref for Channel<T> {
    type Target = Arc<ChannelInner<T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Clone)]
pub struct Sender<T> {
    inner: Channel<T>,
}

impl<T> Sender<T> {
    pub fn send(&self, item: T) {
        // add item to queue
        let mut lock = self.inner.data.lock().unwrap();
        lock.push_back(item);
        drop(lock);
        // wake up waiting thread
    }

    pub fn send_many(&self, items: &[T]) {
        // add item to queue
        let mut lock = self.inner.data.lock().unwrap();
        for item in items {
            lock.push_back(item);
        }
        drop(lock);
        // wake up waiting thread
    }
}

#[derive(Clone)]
pub struct Receiver<T> {
    inner: Channel<T>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Option<T> {
        todo!()
    }

    pub fn recv_exact(&self, num: usize) -> Option<Vec<T>> {
        todo!()
    }

    pub fn recv_all(&self) -> Option<Vec<T>> {
        todo!()
    }
}

unsafe impl<T> Sync for Sender<T> {}
unsafe impl<T> Send for Sender<T> {}

unsafe impl<T> Sync for Receiver<T> {}
unsafe impl<T> Send for Receiver<T> {}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let (tx, rx) = channel();
        tx.send(101);
        assert_eq!(rx.recv().unwrap(), 101);
    }
}
