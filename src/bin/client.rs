use std::{env, io, net::TcpStream};

const HELP: &str = "
aqmm-client - A Quick Market Maker Client

USAGE:
    aqmm-client IP
";

fn run_client(addr: String) -> io::Result<()> {
    let _conn = TcpStream::connect(addr);
    Ok(())
}

fn main() -> io::Result<()> {
    pretty_env_logger::init_timed();

    let mut env = env::args().skip(1).take(2);
    if let Some(ip) = env.next() {
        return run_client(ip);
    }
    eprintln!("{}", HELP);
    Ok(())
}
