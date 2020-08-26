pub(crate) mod context;

use std::{
    io,
    sync::{Arc, Weak},
};

use crate::io::IoState;

use mio::{Interest, Token};
use parking_lot::Mutex;
use slab::Slab;

const CAPACITY: usize = 1024;

pub(crate) struct Driver {
    events: mio::Events,
    inner: Arc<Inner>,
}

#[derive(Clone)]
pub(crate) struct Handle {
    inner: Weak<Inner>,
}

pub(crate) struct Inner {
    pub(crate) poll: Mutex<mio::Poll>,
    pub(crate) states: Mutex<Slab<Arc<IoState>>>,
}

impl Driver {
    pub(crate) fn new() -> io::Result<Self> {
        mio::Poll::new().map(|poll| Self {
            events: mio::Events::with_capacity(CAPACITY / 8),
            inner: Arc::new(Inner {
                states: Mutex::new(Slab::with_capacity(CAPACITY)),
                poll: Mutex::new(poll),
            }),
        })
    }
    pub(crate) fn handle(&self) -> Handle {
        let inner = Arc::downgrade(&self.inner);
        Handle { inner }
    }
    pub(crate) fn turn(&mut self, timeout: Option<std::time::Duration>) -> io::Result<()> {
        match self.inner.poll.try_lock() {
            Some(mut poll) => poll.poll(&mut self.events, timeout)?,
            None => return Ok(()),
        }
        let states = self.inner.states.lock();

        for event in self.events.iter() {
            let state = states.get(event.token().0).ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "driver entry was not found")
            })?;
            if event.is_readable() {
                state.register_read();
            }
            if event.is_writable() {
                state.register_write();
            }
        }
        Ok(())
    }
}

impl Handle {
    pub(crate) fn upgrade(&self) -> Option<Arc<Inner>> {
        self.inner.upgrade()
    }
    pub(crate) fn register(
        &self,
        shared: Arc<IoState>,
        io: &mut dyn mio::event::Source,
    ) -> Option<io::Result<Token>> {
        self.upgrade().map(|inner| {
            let token = Token(inner.states.lock().insert(shared));
            inner
                .poll
                .lock()
                .registry()
                .register(io, token, Interest::READABLE | Interest::WRITABLE)
                .map(|_| token)
        })
    }
}
