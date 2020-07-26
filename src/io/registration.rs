use std::io;
use std::sync::{
    atomic::{AtomicBool, Ordering::SeqCst},
    Arc,
};
use std::task::{Context, Poll};

use crate::io::driver::{self, context};

use futures::task::AtomicWaker;

struct State(AtomicWaker, AtomicBool);

pub(crate) struct IoState {
    read: State,
    write: State,
}

pub(crate) struct Registration<E: mio::event::Source> {
    io: E,
    handle: driver::Handle,
    token: mio::Token,
    shared: Arc<IoState>,
}

impl IoState {
    pub(crate) fn register_read(&self) {
        self.read.1.store(true, SeqCst);
        self.read.0.wake();
    }
    pub(crate) fn register_write(&self) {
        self.write.1.store(true, SeqCst);
        self.write.0.wake();
    }
}

impl<E: mio::event::Source> Registration<E> {
    pub(crate) fn new(mut io: E) -> io::Result<Self> {
        let handle = if let Some(handle) = context::current() {
            handle
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "unable to get io handle",
            ));
        };
        let shared = Arc::new(IoState {
            read: State(AtomicWaker::new(), AtomicBool::new(false)),
            write: State(AtomicWaker::new(), AtomicBool::new(false)),
        });
        let token = handle
            .register(shared.clone(), &mut io)
            .transpose()?
            .expect("unable to access io handle");
        Ok(Self {
            io,
            shared,
            handle,
            token,
        })
    }
    pub(crate) fn io(&mut self) -> &mut E {
        &mut self.io
    }
}

impl<E: mio::event::Source + Unpin> Registration<E> {
    pub(crate) fn poll_read(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if self.shared.read.1.compare_and_swap(true, false, SeqCst) {
            return Poll::Ready(());
        }
        self.shared.read.0.register(cx.waker());
        Poll::Pending
    }
    pub(crate) fn clear_read(&mut self, cx: &mut Context<'_>) {
        self.shared.read.1.store(false, SeqCst);
        self.shared.read.0.register(cx.waker());
    }
    pub(crate) fn poll_write(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if self.shared.write.1.load(SeqCst) {
            return Poll::Ready(());
        }
        self.shared.write.0.register(cx.waker());
        Poll::Pending
    }
    pub(crate) fn clear_write(&mut self, cx: &mut Context<'_>) {
        self.shared.write.1.store(false, SeqCst);
        self.shared.write.0.register(cx.waker());
    }
}

impl<E: mio::event::Source> Drop for Registration<E> {
    fn drop(&mut self) {
        let inner = match self.handle.upgrade() {
            Some(inner) => inner,
            None => return,
        };
        inner.states.borrow_mut().remove(self.token.0);
        let _ = inner.poll.borrow().registry().deregister(&mut self.io);
    }
}
impl Drop for State {
    fn drop(&mut self) {
        self.0.wake();
    }
}
