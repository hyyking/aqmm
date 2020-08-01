use std::{
    cell::RefCell, collections::VecDeque, future::Future, io, net::ToSocketAddrs, time::Duration,
};

use crate::{
    io::driver::{self, context},
    market::{self, securities},
    net::{codec, TcpStream},
    protocol::{
        request::Request::{Auth, Order, Security},
        response, Response, ResponseAuthenticate, ResponseSecurities,
    },
};

const QUEUE_SIZE: usize = 512;

thread_local! {
    static QUEUE: RefCell<VecDeque<async_task::Task<()>>> = RefCell::new(VecDeque::with_capacity(QUEUE_SIZE));
}

// Logarithmic Scoring Rule Market
fn lsrm(entries: &[market::pool::Entry]) -> f64 {
    const B: f64 = 1f64;
    let qtt = entries.iter().map(|entry| entry.quantity);
    B * qtt.fold(0f64, |sum, el| sum + (el / B).exp()).ln()
}

fn spawn<F: Future + 'static>(f: F) {
    let schedule = |task| {
        QUEUE.with(|q| {
            q.borrow_mut().push_back(task);
        })
    };
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
            market: market::Market::new("224.0.0.0:4000", lsrm, 2)?,
        })
    }
    pub fn run(&mut self) -> io::Result<()> {
        let market = self.market.clone();
        context::enter(self.driver.handle(), || loop {
            QUEUE.with(|q| {
                let mut queue = q.borrow_mut();
                for _ in 0..3 {
                    while let Some(task) = queue.pop_front() {
                        let _: bool = task.run();
                    }
                }
            });

            self.driver
                .turn(Some(Duration::from_micros(1)), &mut self.buffer)?;
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
    let addr = stream.get_ref().get_ref().ip();

    spawn(async move {
        while let Some(Ok(req)) = stream.next().await {
            let id = req.id;
            let uuid = req.uuid;

            let res = match req.request {
                Some(Auth(_)) => {
                    debug!("Auth request from {:?}", addr);

                    let uuid = match uuid.iter().sum() {
                        0 => {
                            warn!("create entry in db with new uuid");
                            uuid::Uuid::nil()
                        }
                        _ => {
                            warn!("read account in database");
                            uuid::Uuid::nil()
                        }
                    };
                    Some(Response {
                        id,
                        response: Some(response::Response::Auth(ResponseAuthenticate {
                            uuid: Vec::from(&uuid.as_bytes()[..]),
                        })),
                    })
                }
                Some(Security(_secs)) => {
                    debug!("Security list request from {:?}", addr);
                    Some(Response {
                        id,
                        response: Some(response::Response::Security(ResponseSecurities {
                            securities: securities::as_vec(),
                        })),
                    })
                }
                Some(Order(req)) => {
                    debug!("Order request from {:?}", addr);

                    let response = market.forward(req);

                    // TODO: remove this delay
                    std::thread::sleep(Duration::from_millis(1));

                    Some(Response {
                        id,
                        response: Some(response::Response::Order(response.await.unwrap())),
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
