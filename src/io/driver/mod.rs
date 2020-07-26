pub(crate) mod context;

use std::{
    cell::RefCell,
    convert::TryInto,
    io,
    io::ErrorKind::WouldBlock,
    net::ToSocketAddrs,
    sync::{Arc, Weak},
};

use crate::{io::IoState, net::TcpStream};

use mio::{event::Event, net::TcpListener, Interest, Token};
use slab::Slab;

const CAPACITY: usize = 1024;

pub struct Driver {
    events: mio::Events,
    listener: TcpListener,
    inner: Arc<Inner>,
}

#[derive(Clone)]
pub struct Handle {
    inner: Weak<Inner>,
}

pub(crate) struct Inner {
    pub(crate) poll: RefCell<mio::Poll>,
    pub(crate) states: RefCell<Slab<Arc<IoState>>>,
}

impl Driver {
    const LISTENER: Token = Token(4096);

    pub fn bind<R: ToSocketAddrs>(addr: R) -> io::Result<Self> {
        let addr = addr.to_socket_addrs().unwrap().next().unwrap();
        let poll = mio::Poll::new()?;
        let mut listener = TcpListener::bind(addr)?;
        poll.registry()
            .register(&mut listener, Self::LISTENER, Interest::READABLE)?;
        Ok(Self {
            events: mio::Events::with_capacity(CAPACITY / 2),
            listener,
            inner: Arc::new(Inner {
                poll: RefCell::new(poll),
                states: RefCell::new(Slab::with_capacity(CAPACITY)),
            }),
        })
    }
    pub fn handle(&self) -> Handle {
        let inner = Arc::downgrade(&self.inner);
        Handle { inner }
    }
    pub fn turn(
        &mut self,
        timeout: Option<std::time::Duration>,
        buffer: &mut Vec<TcpStream>,
    ) -> io::Result<usize> {
        self.inner
            .poll
            .borrow_mut()
            .poll(&mut self.events, timeout)?;

        for event in &self.events {
            if event.token() == Self::LISTENER {
                loop {
                    let (stream, _) = match self.listener.accept() {
                        Ok(conn) => conn,
                        Err(ref e) if e.kind() == WouldBlock => break, // no more connections
                        Err(err) => return Err(err),
                    };
                    info!("new connection from {:?}", stream.peer_addr().unwrap());
                    buffer.push(stream.try_into()?);
                }
            } else {
                self.dispatch(event)?;
            }
        }
        Ok(buffer.len())
    }
    pub fn dispatch(&self, event: &Event) -> io::Result<()> {
        let inner = self.inner.states.borrow();
        let state = inner.get(event.token().0).unwrap();

        if event.is_readable() {
            state.register_read();
        }
        if event.is_writable() {
            state.register_write();
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
        match self.upgrade() {
            Some(inner) => {
                let token = Token(inner.states.borrow_mut().insert(shared));
                Some(
                    inner
                        .poll
                        .borrow()
                        .registry()
                        .register(io, token, Interest::READABLE | Interest::WRITABLE)
                        .map(|_| token),
                )
            }
            None => None,
        }
    }
}
