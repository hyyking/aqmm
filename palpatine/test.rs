#![feature(try_blocks)]

use futures_util::stream::StreamExt;
use palpatine::{
    executor::{spawn, Executor},
    net::{TcpListener, TcpStream},
};
use std::io;

fn main() -> io::Result<()> {
    let mut exec = Executor::new()?;
    exec.block_on(async {
        for _ in 0..10 {
            spawn(async {
                let _stream = TcpStream::connect("127.0.0.1:8000").unwrap();
            });
        }

        let listener = TcpListener::bind("127.0.0.1:8000")?;
        let mut listener = listener.take(10);
        while let Some(Ok(stream)) = listener.next().await {
            println!("stream: {:?}", stream.peer_addr().unwrap());
        }
        Ok(())
    })
}
