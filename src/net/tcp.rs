use std::{
    convert::TryFrom,
    io::{self, ErrorKind::WouldBlock, Read, Write},
    pin::Pin,
    task::{Context, Poll},
};

use crate::io::Registration;

use futures::{ready, AsyncRead, AsyncWrite};
use mio::net;

pub struct TcpStream {
    inner: Registration<net::TcpStream>,
}

impl TcpStream {
    pub(crate) fn ip(&self) -> std::net::SocketAddr {
        self.inner.io_ref().peer_addr().unwrap()
    }
}

impl TryFrom<net::TcpStream> for TcpStream {
    type Error = io::Error;

    fn try_from(io: net::TcpStream) -> Result<Self, Self::Error> {
        let inner = Registration::new(io)?;
        Ok(Self { inner })
    }
}

impl AsyncRead for TcpStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        ready!(self.inner.poll_read(cx));

        let res = self.inner.io().read(buf);
        if matches!(res.as_ref().map_err(io::Error::kind), Err(WouldBlock)) {
            self.inner.clear_read(cx);
            return Poll::Pending;
        }
        Poll::Ready(res)
    }
}
impl AsyncWrite for TcpStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        ready!(self.inner.poll_write(cx));

        let res = self.inner.io().write(buf);
        if matches!(res.as_ref().map_err(io::Error::kind), Err(WouldBlock)) {
            self.inner.clear_write(cx);
            return Poll::Pending;
        }
        Poll::Ready(res)
    }
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
    fn poll_close(mut self: Pin<&mut Self>, _: &mut Context) -> Poll<io::Result<()>> {
        Poll::Ready(self.inner.io().shutdown(std::net::Shutdown::Write))
    }
}
