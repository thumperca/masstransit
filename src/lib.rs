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
        let num_items = lock.len();
        drop(lock);
        // wake up waiting thread
        let mut taken = 0;
        let mut lock = self.inner.queue.lock().unwrap();
        while let Some(wt) = lock.pop_front() {
            if wt.count == usize::MAX {
                taken = usize::MAX;
            } else {
                taken += wt.count;
            }
            wt.thread.unpark();
            if taken >= num_items {
                break;
            }
        }
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
        if self.inner.num_senders.fetch_sub(1, Release) == 1 {
            let lock = self.inner.queue.lock().unwrap();
            for item in lock.iter() {
                item.thread.unpark();
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
        loop {
            let mut lock = self.inner.data.lock().unwrap();
            // there's an item in the queue
            let mut data = Vec::with_capacity(num);
            loop {
                if let Some(item) = lock.pop_front() {
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
            drop(lock);
            // channel is closed
            if self.inner.num_senders.load(Acquire) == 0 {
                return None;
            }
            // queue is empty
            let wt = WaitingThread::new(num);
            let mut lock = self.inner.queue.lock().unwrap();
            lock.push_back(wt);
            drop(lock);
            std::thread::park();
        }
    }

    pub fn recv_all(&self) -> Option<Vec<T>> {
        loop {
            let mut lock = self.inner.data.lock().unwrap();
            let data = std::mem::take(&mut *lock);
            if !data.is_empty() {
                let data = Vec::from_iter(data.into_iter());
                return Some(data);
            } else if self.inner.num_senders.load(Acquire) == 0 {
                return None;
            }
            // wait for messages
            let wt = WaitingThread::new(usize::MAX);
            let mut lock = self.inner.queue.lock().unwrap();
            lock.push_back(wt);
            drop(lock);
            std::thread::park();
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
