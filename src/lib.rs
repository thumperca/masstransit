use std::collections::VecDeque;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

struct ChannelInner<T> {
    lock: AtomicU32,
    data: VecDeque<T>,
}

impl<T> ChannelInner<T> {
    fn new() -> Self {
        Self {
            lock: AtomicU32::new(0),
            data: VecDeque::new(),
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

#[derive(Clone)]
pub struct Sender<T> {
    inner: Channel<T>,
}

impl<T> Sender<T> {
    pub fn send(&self, item: T) {
        todo!()
    }

    pub fn bulk_send(&self, items: &[T]) {
        todo!()
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

    pub fn bulk_recv(&self) -> Option<Vec<T>> {
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
