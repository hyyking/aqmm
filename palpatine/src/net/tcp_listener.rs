use std::{
    convert::TryFrom,
    io::{self, ErrorKind::WouldBlock},
    net::{SocketAddr, ToSocketAddrs},
    pin::Pin,
    task::{Context, Poll},
};

use crate::{io::Registration, net::TcpStream};

use futures_util::{future, stream::Stream};
use mio::net;

pub struct TcpListener {
    inner: Registration<net::TcpListener>,
}

impl TcpListener {
    /// # Errors
    /// Fails if no address could be bound or I/O driver is not in scope
    pub fn bind<S: ToSocketAddrs>(addr: S) -> io::Result<Self> {
        addr.to_socket_addrs()?.fold(
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "empty list of adresses",
            )),
            |opt, addr| {
                opt.or_else(|_| {
                    net::TcpListener::bind(addr).and_then(|listener| {
                        Ok(Self {
                            inner: Registration::new(listener)?,
                        })
                    })
                })
            },
        )
    }

    /// # Errors
    /// See [`io::Result`](std::io::Result), this method will not error if the socket returns
    /// [`WouldBlock`](std::io::ErrorKind::WouldBlock)
    pub async fn accept(&mut self) -> io::Result<(TcpStream, SocketAddr)> {
        future::poll_fn(|cx| self.poll_accept(cx)).await
    }

    fn poll_accept(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<(TcpStream, SocketAddr)>> {
        ready!(self.inner.poll_read(cx));

        let res = self.inner.io_ref().accept();
        if matches!(res.as_ref().map_err(io::Error::kind), Err(WouldBlock)) {
            self.inner.clear_read(cx);
            return Poll::Pending;
        }
        Poll::Ready(res.and_then(|(tcp, addr)| Ok((TcpStream::try_from(tcp)?, addr))))
    }
}

impl Stream for TcpListener {
    type Item = io::Result<TcpStream>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let (socket, _) = ready!(self.poll_accept(cx))?;
        Poll::Ready(Some(Ok(socket)))
    }
}
