use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

use crate::{
    net::codec::{self, Error},
    protocol::{Request, Response},
};

use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

pub struct Client {
    uuid: Option<u64>,
    stream: TcpStream,
    buffer: BytesMut,
}

impl Client {
    pub fn connect<S: ToSocketAddrs>(addr: S) -> io::Result<Self> {
        Ok(Self {
            uuid: None,
            stream: TcpStream::connect(addr)?,
            buffer: BytesMut::with_capacity(512),
        })
    }
    pub fn is_connected(&self) -> bool {
        self.uuid.is_some()
    }
    pub fn send(&mut self, req: Request) -> io::Result<Option<Response>> {
        let mut codec = codec::Client;

        #[allow(clippy::single_match)]
        match codec.encode(req, &mut self.buffer) {
            Err(Error::Io(io)) => return Err(io),
            _ => { /* TODO: handle buffer resizing */ }
        }

        self.stream.write_all(&self.buffer)?;
        self.stream.flush()?;
        trace!("sent request: {} bytes", self.buffer.len());

        self.buffer.clear();
        let buf = unsafe {
            let buf = self.buffer.bytes_mut();
            buf.iter_mut().for_each(|el| {
                el.write(0);
            });
            std::mem::MaybeUninit::slice_get_mut(buf)
        };

        let n = self.stream.read(buf)?;
        unsafe { self.buffer.advance_mut(n) };
        trace!("read response: {} bytes", n);

        if n == 0 {
            return match codec.decode_eof(&mut self.buffer) {
                Ok(e) => Ok(e),
                Err(Error::Io(io)) => Err(io),
                Err(e) => {
                    error!("{:?}", e);
                    Ok(None)
                }
            };
        }
        if let Ok(Some(message)) = codec.decode(&mut self.buffer) {
            Ok(Some(message))
        } else {
            warn!("unable to decode message");
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}
