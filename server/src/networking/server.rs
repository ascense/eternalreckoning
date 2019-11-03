use std::sync::{
    Arc,
    Mutex,
};

use failure::{
    format_err,
    Error,
};
use tokio::net::{
    UdpSocket,
    UdpFramed,
};
use tokio::prelude::*;

use eternalreckoning_core::net::codec::EternalReckoningCodec;

use super::{
    error::NetworkError,
    state::{
        State,
        SharedState,
    },
    reader::{
        Reader,
        Tx,
    },
    writer::{
        Writer,
        Rx,
    },
};

pub struct Server {
    state: SharedState,
}

impl Server {
    pub fn new() -> Server {
        Server {
            state: Arc::new(Mutex::new(State::new())),
        }
    }

    pub fn run(
        self,
        address: &String,
        rx: Rx,
        tx: Tx,
    )
    {
        let addr = address.parse().unwrap();
        let socket = UdpSocket::bind(&addr).unwrap();
        log::info!("Listening on: {}", &addr);

        let server = ServerFuture::new(&self.state, socket, tx, rx);

        tokio::run(
            server
                .map_err(|err| {
                    log::error!("Fatal error: {}", err)
                })
                .map(|_| {
                    log::info!("Closing server")
                })
        );
    }
}

struct ServerFuture {
    reader: Reader,
    writer: Writer,
}

impl ServerFuture {
    pub fn new(state: &SharedState, socket: UdpSocket, tx: Tx, rx: Rx)
        -> ServerFuture
    {
        let framed = UdpFramed::new(socket, EternalReckoningCodec);
        let (sink, stream) = framed.split();

        ServerFuture {
            reader: Reader::new(state.clone(), stream, tx),
            writer: Writer::new(state.clone(), sink, rx),
        }
    }

    fn map_result(result: Poll<(), NetworkError>) -> Poll<(), Error> {
        match result {
            Ok(result) => Ok(result),
            Err(NetworkError::RebuildRequired) => Ok(Async::Ready(())),
            Err(NetworkError::FatalError(err)) => Err(err),
        }
    }

    fn join_result(lhs: Poll<(), Error>, rhs: Poll<(), Error>) -> Poll<(), Error> {
        if lhs.is_err() {
            return lhs;
        }
        if rhs.is_err() {
            return rhs;
        }

        if lhs.unwrap() == Async::Ready(()) || rhs.unwrap() == Async::Ready(()) {
            return Ok(Async::Ready(()));
        }

        Ok(Async::NotReady)
    }
}

impl Future for ServerFuture {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Error> {
        let result = Self::join_result(
            Self::map_result(self.reader.poll()),
            Self::map_result(self.writer.poll())
        );

        match result {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(err) => Err(err),
            Ok(Async::Ready(())) => {
                return Err(format_err!("Server socket unexpectedly lost"));
            },
        }
    }
}