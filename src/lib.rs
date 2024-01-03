use atomic_wait::{wait, wake_all, wake_one};
use crossbeam::queue::SegQueue;
use std::ops::Deref;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::Arc;

struct Counter {
    senders: AtomicU32,
    receivers: AtomicU32,
    waiting: AtomicU32,
}

struct ChannelInner<T> {
    // u32 for threads to wait on
    wait: AtomicU32,
    // channel messages
    data: SegQueue<T>,
    // number of senders to keep track when channel is closed
    counter: Counter,
}

impl<T> ChannelInner<T> {
    fn new() -> Self {
        Self {
            wait: AtomicU32::new(0),
            data: SegQueue::new(),
            counter: Counter {
                senders: AtomicU32::new(1),
                receivers: AtomicU32::new(1),
                waiting: AtomicU32::new(0),
            },
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

pub struct Sender<T> {
    inner: Channel<T>,
}

impl<T> Sender<T> {
    pub fn send(&self, item: T) {
        // add item to queue
        self.inner.data.push(item);
        // wake up waiting thread
        self.inner.wait.fetch_add(1, Release);
        if self.inner.counter.waiting.load(Acquire) > 0 {
            wake_one(&self.inner.wait);
        }
    }

    pub fn send_many(&self, items: Vec<T>) {
        // add item to queue
        for item in items {
            self.inner.data.push(item);
        }
        let num_items = self.inner.data.len();
        self.inner.wait.fetch_add(1, Release);
        // wake up waiting thread
        if self.inner.counter.waiting.load(Acquire) > 0 {
            if num_items == 1 {
                wake_one(&self.inner.wait);
            } else {
                wake_all(&self.inner.wait);
            }
        }
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.inner.counter.senders.fetch_add(1, Relaxed);
        let channel = Channel {
            inner: self.inner.clone(),
        };
        Self { inner: channel }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        if self.inner.counter.senders.fetch_sub(1, Release) == 1 {
            if self.inner.counter.waiting.load(Acquire) != 0 {
                wake_all(&self.inner.wait);
            }
        }
    }
}

#[derive(Clone)]
pub struct Receiver<T> {
    inner: Channel<T>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Option<T> {
        loop {
            // there's an item in the queue
            if let Some(item) = self.inner.data.pop() {
                return Some(item);
            }
            // channel is closed
            let num_senders = self.inner.counter.senders.load(Acquire);
            if num_senders == 0 {
                return None;
            }
            // queue is empty
            self.inner.counter.waiting.fetch_add(1, Release);
            wait(&self.inner.wait, self.inner.wait.load(Relaxed));
            self.inner.counter.waiting.fetch_sub(1, Release);
        }
    }

    pub fn recv_exact(&self, num: usize) -> Option<Vec<T>> {
        loop {
            // there's an item in the queue
            let mut data = Vec::with_capacity(num);
            loop {
                if let Some(item) = self.inner.data.pop() {
                    data.push(item);
                    if data.len() == num {
                        return Some(data);
                    }
                } else if !data.is_empty() {
                    return Some(data);
                } else {
                    break;
                }
            }
            // channel is closed
            if self.inner.counter.senders.load(Acquire) == 0 {
                return None;
            }
            // queue is empty
            self.inner.counter.waiting.fetch_add(1, Release);
            wait(&self.inner.wait, self.inner.wait.load(Relaxed));
            self.inner.counter.waiting.fetch_sub(1, Release);
        }
    }

    pub fn recv_all(&self) -> Option<Vec<T>> {
        loop {
            // there's an item in the queue
            let mut data = Vec::new();
            loop {
                if let Some(item) = self.inner.data.pop() {
                    data.push(item);
                } else if !data.is_empty() {
                    return Some(data);
                } else {
                    break;
                }
            }
            // channel is closed
            if self.inner.counter.senders.load(Acquire) == 0 {
                return None;
            }
            // queue is empty
            self.inner.counter.waiting.fetch_add(1, Release);
            wait(&self.inner.wait, self.inner.wait.load(Relaxed));
            self.inner.counter.waiting.fetch_sub(1, Release);
        }
    }
}

unsafe impl<T> Sync for Sender<T> {}
unsafe impl<T> Send for Sender<T> {}

unsafe impl<T> Sync for Receiver<T> {}
unsafe impl<T> Send for Receiver<T> {}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let channel = Channel::new();
    let sender = Sender {
        inner: Channel {
            inner: channel.inner.clone(),
        },
    };
    let recv = Receiver { inner: channel };
    (sender, recv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn it_works() {
        let (tx, rx) = channel();
        tx.send(101);
        assert_eq!(rx.recv().unwrap(), 101);
    }

    #[test]
    fn channel_close() {
        let (tx, rx) = channel();
        tx.send(101);
        assert_eq!(rx.recv().unwrap(), 101);
        drop(tx);
        assert!(rx.recv().is_none());
    }

    #[test]
    fn wake_up_waiting_thread() {
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_micros(100));
            tx.send(101);
        });
        assert_eq!(rx.recv().unwrap(), 101);
        assert!(rx.recv().is_none());
    }

    #[test]
    fn recv_exact() {
        let data = (0..8).collect::<Vec<u16>>();
        let (tx, rx) = channel();
        tx.send_many(data);
        drop(tx);
        assert_eq!(rx.recv_exact(5).unwrap().len(), 5);
        assert_eq!(rx.recv_exact(5).unwrap().len(), 3);
        assert!(rx.recv_exact(5).is_none());
    }

    #[test]
    fn recv_all() {
        let data = (0..8).collect::<Vec<u16>>();
        let (tx, rx) = channel();
        tx.send_many(data);
        drop(tx);
        assert_eq!(rx.recv_all().unwrap().len(), 8);
        assert!(rx.recv_all().is_none());
    }
}
