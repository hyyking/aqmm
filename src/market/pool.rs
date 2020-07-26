use std::io;
use std::net::{self, ToSocketAddrs, UdpSocket};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use super::securities;

#[derive(Clone, Copy)]
pub struct Entry {
    pub(crate) id: usize,
    pub(crate) quantity: f64,
}

pub struct SecurityPool {
    securities: Mutex<Box<[Entry]>>,
    _broadcast: Broadcaster,
}

struct Broadcaster {
    _prev: Instant,
    _socket: UdpSocket,
}

impl SecurityPool {
    pub(crate) fn new(addr: impl ToSocketAddrs) -> io::Result<Self> {
        let mut securities = securities::map_vec(|s| Entry {
            id: s.security_id as usize,
            quantity: 0f64,
        });
        securities.sort_unstable_by_key(|s| s.id);

        let addr = addr.to_socket_addrs().unwrap().next().unwrap();
        validate_address(&addr)?;
        let socket = UdpSocket::bind(addr)?;

        let prev = Instant::now();
        Ok(Self {
            securities: Mutex::new(securities.into_boxed_slice()),
            _broadcast: Broadcaster {
                _prev: prev,
                _socket: socket,
            },
        })
    }

    pub(crate) fn access(&self) -> MutexGuard<'_, Box<[Entry]>> {
        self.securities.lock().unwrap()
    }

    pub(crate) fn broadcast(&self, _timeout: Option<Duration>) -> io::Result<()> {
        Ok(())
    }
}

fn validate_address(addr: &net::SocketAddr) -> io::Result<()> {
    match addr {
        &net::SocketAddr::V6(_) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "no support for ipv6",
            ))
        }
        &net::SocketAddr::V4(ref addr) => {
            if !addr.ip().is_multicast() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "expected a broadcast address",
                ));
            }
        }
    };
    Ok(())
}
