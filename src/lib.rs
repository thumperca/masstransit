use std::collections::VecDeque;

struct Channel<T> {
    data: VecDeque<T>,
}

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

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    todo!()
}
