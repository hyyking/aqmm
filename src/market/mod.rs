mod core;
pub(crate) mod pool;
mod router;
pub(crate) mod securities;

use std::io;
use std::net::ToSocketAddrs;
use std::rc::Rc;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crate::protocol::{RequestOrder, ResponseOrder};

use futures::channel::oneshot::{self, Receiver};

#[derive(Clone)]
pub struct Market {
    inner: Rc<Inner>,
}

struct Inner {
    router: router::Router<core::Task>,
    cores: Option<Box<[JoinHandle<()>]>>,
}

impl Market {
    pub(crate) fn new(
        addr: impl ToSocketAddrs,
        score: core::ScoreFn,
        cores: usize,
    ) -> io::Result<Self> {
        let pool = Arc::new(pool::SecurityPool::new(addr)?);
        let mut router = router::Router::with_capacity(cores);
        let c = cores;

        let mut cores = Vec::with_capacity(cores);

        for _ in 0..c {
            let core = core::Core {
                score,
                pool: pool.clone(),
                receiver: router.receiver(),
            };
            cores.push(thread::spawn(|| {
                futures::executor::block_on(core::run(core))
            }));
        }

        Ok(Self {
            inner: Rc::new(Inner {
                router,
                cores: Some(cores.into_boxed_slice()),
            }),
        })
    }

    pub(crate) fn forward(&self, req: RequestOrder) -> Receiver<ResponseOrder> {
        let (sender, rcv) = oneshot::channel();
        self.inner.router.send(core::Task { req, sender });
        rcv
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        if let Some(cores) = self.cores.take() {
            for h in Vec::from(cores).drain(..) {
                let _ = h.join();
            }
        }
    }
}
