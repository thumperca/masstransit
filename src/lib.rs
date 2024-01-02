use std::collections::VecDeque;
use std::ops::Deref;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::{Arc, Mutex};
use std::thread::Thread;

struct WaitingThread {
    // handle to the waiting thread to wake it up
    thread: Thread,
    // number of items the thread is waiting for
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
    // queue of waiting threads
    queue: Mutex<VecDeque<WaitingThread>>,
    // channel messages
    data: Mutex<VecDeque<T>>,
    // number of senders to keep track when channel is closed
    num_senders: AtomicU32,
}

impl<T> ChannelInner<T> {
    fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            data: Mutex::new(VecDeque::new()),
            num_senders: AtomicU32::new(1),
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
        let mut lock = self.inner.data.lock().unwrap();
        lock.push_back(item);
        drop(lock);
        // wake up waiting thread
        let mut lock = self.inner.queue.lock().unwrap();
        if let Some(wt) = lock.pop_front() {
            wt.thread.unpark();
        }
    }

    pub fn send_many(&self, items: Vec<T>) {
        // add item to queue
        let mut lock = self.inner.data.lock().unwrap();
        for item in items {
            lock.push_back(item);
        }
        drop(lock);
        // wake up waiting thread
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.inner.num_senders.fetch_add(1, Relaxed);
        let channel = Channel {
            inner: self.inner.clone(),
        };
        Self { inner: channel }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.inner.num_senders.fetch_sub(1, Release);
    }
}

#[derive(Clone)]
pub struct Receiver<T> {
    inner: Channel<T>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Option<T> {
        loop {
            let mut lock = self.inner.data.lock().unwrap();
            let pop_result = lock.pop_front();
            drop(lock);
            // there's an item in the queue
            if let Some(item) = pop_result {
                return Some(item);
            }
            // channel is closed
            if self.inner.num_senders.load(Acquire) == 0 {
                return None;
            }
            // queue is empty
            let wt = WaitingThread::new(1);
            let mut lock = self.inner.queue.lock().unwrap();
            lock.push_back(wt);
            drop(lock);
            std::thread::park();
        }
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
}
