#![feature(never_type)]

use aqmm::Server;

use std::{env, io};

const HELP: &str = "
aqmm-server - A Quick Market Maker Server

USAGE:
    aqmm-server IP
";

fn run_server(addr: String) -> io::Result<()> {
    let mut server = Server::new(addr)?;
    server.run()
}

fn main() -> io::Result<()> {
    pretty_env_logger::init_timed();

    let mut env = env::args().skip(1).take(1);
    if let Some(ip) = env.next() {
        return run_server(ip).map(|_| ());
    }
    eprintln!("{}", HELP);
    Ok(())
}
