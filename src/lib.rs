#![feature(maybe_uninit_extra, maybe_uninit_slice_assume_init)]

#[macro_use]
extern crate log;

pub mod protocol {
    include!(concat!(env!("OUT_DIR"), "/protocol.rs"));
}
mod client;
mod io;
mod market;
mod net;
mod server;

pub use client::Client;
pub use server::Server;
