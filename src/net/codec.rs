use crate::protocol::{Request, Response};

use bytes::BytesMut;
use prost::Message as _;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Decode(prost::DecodeError),
    Encode(prost::EncodeError),
}
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Copy)]
pub struct Server;

impl Decoder for Server {
    type Item = Request;
    type Error = Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }
        Request::decode_length_delimited(src)
            .map(Some)
            .map_err(Error::Decode)
    }
}
impl Encoder<Response> for Server {
    type Error = Error;
    fn encode(&mut self, item: Response, dst: &mut BytesMut) -> Result<(), Self::Error> {
        Response::encode_length_delimited(&item, dst).map_err(Error::Encode)
    }
}

#[derive(Clone, Copy)]
pub struct Client;

impl Decoder for Client {
    type Item = Response;
    type Error = Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }
        Response::decode_length_delimited(src)
            .map(Some)
            .map_err(Error::Decode)
    }
}
impl Encoder<Request> for Client {
    type Error = Error;
    fn encode(&mut self, item: Request, dst: &mut BytesMut) -> Result<(), Self::Error> {
        Request::encode_length_delimited(&item, dst).map_err(Error::Encode)
    }
}
