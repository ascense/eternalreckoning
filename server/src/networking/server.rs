use std::sync::{
    Arc,
    Mutex,
};

use failure::format_err;
use tokio::net::{
    UdpSocket,
    UdpFramed,
};
use tokio::prelude::*;

use eternalreckoning_core::net::codec::EternalReckoningCodec;

use super::{
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
        let framed = UdpFramed::new(
            UdpSocket::bind(&addr).unwrap(),
            EternalReckoningCodec
        );
        log::info!("Listening on: {}", addr);

        let (sink, stream) = framed.split();

        let server_reader = Reader::new(self.state.clone(), stream, tx)
            .map_err(|err| {
                format_err!("Reader error: {}", err)
            });

        let server_writer = Writer::new(self.state.clone(), sink, rx)
            .map_err(|err| {
                format_err!("Writer error: {}", err)
            });

        tokio::run(
            server_reader
                .join(server_writer)
                .map_err(|err| {
                    log::error!("Fatal error: {}", err)
                })
                .map(|_| {
                    log::info!("Closing server")
                })
        );
    }
}