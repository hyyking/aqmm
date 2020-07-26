use std::{cell::RefCell, collections::VecDeque, future::Future, io, net::ToSocketAddrs};

use crate::{
    io::driver::{self, context},
    market::{self, securities},
    net::{codec, TcpStream},
    protocol::{
        request::Request::{Auth, Order, Security},
        response, Response, ResponseAuthenticate, ResponseOrder, ResponseSecurities,
    },
};

use async_task::Task;

thread_local! {
    static QUEUE: RefCell<VecDeque<Task<()>>> = RefCell::new(VecDeque::with_capacity(1024));
}

// Logarithmic Scoring Rule Market
fn lsrm(entries: &[market::pool::Entry]) -> f64 {
    const B: f64 = 1f64;
    let qtt = entries.iter().map(|entry| entry.quantity);
    B * qtt.fold(0f64, |sum, el| sum + (el / B).exp()).ln()
}

fn spawn<F: Future + 'static>(f: F) {
    let schedule = |task| QUEUE.with(|q| q.borrow_mut().push_back(task));
    let (task, _) = async_task::spawn_local(f, schedule, ());
    task.schedule();
}

pub struct Server {
    driver: driver::Driver,
    market: market::Market,
    buffer: Vec<TcpStream>,
}

impl Server {
    pub fn new<R: ToSocketAddrs>(addr: R) -> io::Result<Self> {
        Ok(Self {
            driver: driver::Driver::bind(addr)?,
            buffer: Vec::with_capacity(16),
            market: market::Market::new("224.0.0.0", lsrm, 1)?,
        })
    }
    pub fn run(&mut self) -> io::Result<()> {
        context::enter(self.driver.handle(), || loop {
            QUEUE.with(|q| {
                let mut queue = q.borrow_mut();
                for _ in 0..3 {
                    while let Some(task) = queue.pop_front() {
                        let _: bool = task.run();
                    }
                }
            });

            self.driver.turn(None, &mut self.buffer)?;
            let market = self.market.clone();
            self.buffer
                .drain(..)
                .for_each(|stream| spawn_client(stream, market.clone()));

            // TODO: clean exit
            if false {
                return Ok(());
            }
        })
    }
}

fn spawn_client(stream: TcpStream, market: market::Market) {
    use futures::sink::SinkExt;
    use futures::stream::StreamExt;
    use tokio_util::{codec::Framed, compat::FuturesAsyncReadCompatExt};

    let mut stream = Framed::new(stream.compat(), codec::Server);

    spawn(async move {
        while let Some(Ok(req)) = stream.next().await {
            let id = req.id;
            let _uuid = req.uuid;

            let res = match req.request {
                Some(Auth(_)) => {
                    debug!("Auth request");
                    Some(Response {
                        id,
                        response: Some(response::Response::Auth(ResponseAuthenticate {
                            uuid: vec![0; 16], // TODO: Vec::from(&uuid.as_bytes()[..]),
                        })),
                    })
                }
                Some(Order(req)) => {
                    /* TODO: check the session uuid token */
                    debug!("Order request");

                    market.forward(req);

                    /* TODO: get response */
                    Some(Response {
                        id,
                        response: Some(response::Response::Order(ResponseOrder {
                            orders: vec![/**/],
                        })),
                    })
                }
                Some(Security(_secs)) => {
                    debug!("Security list request");
                    Some(Response {
                        id,
                        response: Some(response::Response::Security(ResponseSecurities {
                            securities: securities::as_vec(),
                        })),
                    })
                }
                None => {
                    warn!("empty request");
                    None
                }
            };
            if let Some(res) = res {
                stream.send(res).await.unwrap();
            }
        }
    })
}
