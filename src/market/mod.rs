mod core;
pub(crate) mod pool;
pub(crate) mod securities;

use std::io;
use std::net::ToSocketAddrs;
use std::sync::Arc;

use crate::protocol::RequestOrder;

#[derive(Clone)]
pub struct Market {
    inner: Arc<Inner>,
}

struct Inner {
    cores: Box<[core::Core]>,
}

impl Market {
    pub(crate) fn new(
        addr: impl ToSocketAddrs,
        score: core::Score,
        cores: usize,
    ) -> io::Result<Self> {
        let securities = Arc::new(pool::SecurityPool::new(addr)?);

        let mut c = Vec::with_capacity(cores);
        c.resize(cores, core::Core::new(score, securities.clone()));

        Ok(Self {
            inner: Arc::new(Inner {
                cores: c.into_boxed_slice(),
            }),
        })
    }

    pub(crate) fn forward(&self, req: RequestOrder) {
        /* TODO: Put a router in place */
        self.inner.cores[0].execute(req);
    }
}
