use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        Arc,
    },
    task::{Context, Poll},
};

use crate::io::driver::{self, context};

use futures_util::task::AtomicWaker;

pub struct Registration<E: mio::event::Source> {
    io: E,
    handle: driver::Handle,
    token: mio::Token,
    state: Arc<IoState>,
}

#[derive(Default)]
pub(crate) struct IoState {
    read: State,
    write: State,
}

#[derive(Default)]
struct State {
    waker: AtomicWaker,
    flag: AtomicBool,
}

impl<E: mio::event::Source> Registration<E> {
    /// # Errors
    /// Fails if the I/O driver is not in scope
    pub fn new(mut io: E) -> io::Result<Self> {
        let handle = if let Some(handle) = context::current() {
            handle
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "unable to get io handle",
            ));
        };
        let state = Arc::new(IoState::default());
        let token = handle
            .register(state.clone(), &mut io)
            .transpose()?
            .expect("unable to access io handle");
        Ok(Self {
            io,
            state,
            handle,
            token,
        })
    }
    pub fn io(&mut self) -> &mut E {
        &mut self.io
    }
    pub fn io_ref(&self) -> &E {
        &self.io
    }

    pub fn poll_read(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if self.state.read.flag.load(SeqCst) {
            Poll::Ready(())
        } else {
            self.state.read.waker.register(cx.waker());
            Poll::Pending
        }
    }
    pub fn clear_read(&mut self, cx: &mut Context<'_>) {
        self.state.read.flag.store(false, SeqCst);
        self.state.read.waker.register(cx.waker());
    }
    pub fn poll_write(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if self.state.write.flag.load(SeqCst) {
            Poll::Ready(())
        } else {
            self.state.read.waker.register(cx.waker());
            Poll::Pending
        }
    }
    pub fn clear_write(&mut self, cx: &mut Context<'_>) {
        self.state.write.flag.store(false, SeqCst);
        self.state.write.waker.register(cx.waker());
    }
}
impl<E: mio::event::Source> Drop for Registration<E> {
    fn drop(&mut self) {
        let inner = match self.handle.upgrade() {
            Some(inner) => inner,
            None => return,
        };
        inner.states.lock().remove(self.token.0);
        let _ = inner.poll.lock().registry().deregister(&mut self.io);
    }
}
impl IoState {
    pub(crate) fn register_read(&self) {
        self.read.flag.store(true, SeqCst);
        if let Some(waker) = self.read.waker.take() {
            waker.wake()
        }
    }
    pub(crate) fn register_write(&self) {
        self.write.flag.store(true, SeqCst);
        if let Some(waker) = self.write.waker.take() {
            waker.wake()
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        self.waker.wake();
    }
}
