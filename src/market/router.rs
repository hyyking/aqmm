use std::{
    cell::Cell,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll},
};

use futures::{stream::Stream, task::AtomicWaker};

pub struct Router<M> {
    wheel: Cell<usize>,
    senders: Vec<Arc<Inner<M>>>,
}
pub struct EndPoint<M> {
    inner: Arc<Inner<M>>,
}
struct Inner<M> {
    lock: AtomicBool,
    data: Cell<Option<M>>,
    waker: AtomicWaker,
}

impl<M> Router<M> {
    pub fn with_capacity(endpoints: usize) -> Self {
        let senders = Vec::with_capacity(endpoints);
        let wheel = Cell::default();
        Self { wheel, senders }
    }
    fn inc(&self) {
        let wheel = self.wheel.take();
        self.wheel.set(wheel + 1);
        if wheel + 1 >= self.senders.len() {
            self.wheel.set(0)
        }
    }
}
impl<M: 'static> Router<M> {
    pub fn receiver(&mut self) -> EndPoint<M> {
        let inner = Arc::new(Inner {
            lock: AtomicBool::new(true),
            data: Cell::new(None),
            waker: AtomicWaker::new(),
        });
        self.senders.push(inner.clone());
        EndPoint { inner }
    }

    pub fn send(&self, mut data: M) {
        let wheel = self.wheel.take();
        while let Err(old_data) = self.senders[wheel].send(data) {
            data = old_data
        }
        self.inc()
    }
}

impl<M> EndPoint<M> {
    pub fn recv(&self) -> Option<M> {
        self.inner.recv()
    }
    fn is_closed(&self) -> bool {
        Arc::strong_count(&self.inner) == 1
    }
}
impl<M> Stream for EndPoint<M> {
    type Item = M;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.inner.waker.register(cx.waker());
        match self.recv() {
            Some(data) => Poll::Ready(Some(data)),
            None if self.is_closed() => Poll::Ready(None),
            None => Poll::Pending,
        }
    }
}
unsafe impl<M> Send for EndPoint<M> {}

impl<M> Inner<M> {
    pub fn send(&self, data: M) -> Result<(), M> {
        if let Some(old_data) = self.data.take() {
            self.data.set(Some(old_data));
            Err(data)
        } else {
            self.data.set(Some(data));
            self.lock.swap(false, Ordering::SeqCst);
            self.waker.wake();
            Ok(())
        }
    }
    pub fn recv(&self) -> Option<M> {
        loop {
            // if current is unlocked, take and relock it
            if !self.lock.compare_and_swap(false, true, Ordering::SeqCst) {
                return self.data.take();
            }
        }
    }
}
