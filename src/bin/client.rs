#[macro_use]
extern crate log;

use std::{env, io};

use aqmm::{
    protocol::{
        request, response::Response::Auth, Kind, Order, Request, RequestAuthenticate, RequestOrder,
        RequestSecurities, ResponseAuthenticate,
    },
    Client,
};

const HELP: &str = "
aqmm-client - A Quick Market Maker Client

USAGE:
    aqmm-client IP
";

fn auth(client: &mut Client) -> io::Result<uuid::Uuid> {
    let res = client
        .send(Request {
            id: 0,
            uuid: vec![],
            request: Some(request::Request::Auth(RequestAuthenticate {})),
        })?
        .unwrap();
    Ok(
        if let Some(Auth(ResponseAuthenticate { uuid })) = res.response {
            let uuid = uuid::Uuid::from_slice(&uuid).unwrap();
            info!("Uuid: {:?}", uuid);
            uuid
        } else {
            error!("unexpected response");
            uuid::Uuid::nil()
        },
    )
}

fn securities(client: &mut Client, uuid: &uuid::Uuid) -> io::Result<()> {
    let res = client
        .send(Request {
            id: 1,
            uuid: Vec::from(&uuid.as_bytes()[..]),
            request: Some(request::Request::Security(RequestSecurities {})),
        })?
        .unwrap();
    info!("{:#?}", res.response.unwrap());
    Ok(())
}

fn run_client(addr: String) -> io::Result<()> {
    let mut client = Client::connect(addr)?;

    let uuid = auth(&mut client)?;
    securities(&mut client, &uuid)?;

    let res = client
        .send(Request {
            id: 2,
            uuid: Vec::from(&uuid.as_bytes()[..]),
            request: Some(request::Request::Order(RequestOrder {
                orders: vec![Order {
                    kind: Kind::Buy as i32,
                    security_id: 0,
                    amount: 1.0,
                }],
            })),
        })?
        .unwrap();
    info!("{:?}", res.response.unwrap());
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
