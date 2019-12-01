use std::net::SocketAddr;
use std::sync::mpsc::Sender;

use failure::{
    format_err,
    Error,
};
use futures::stream::{
    Stream,
    SplitStream,
};
use tokio::net::UdpFramed;
use tokio::prelude::{
    Async,
    Future,
    Poll,
};
use uuid::Uuid;

use eternalreckoning_core::net::{
    codec::EternalReckoningCodec,
    operation::Operation,
};

use super::error::NetworkError;
use super::state::SharedState;

pub type Tx = Sender<(Uuid, Operation)>;

pub struct Reader {
    shared: SharedState,
    stream: SplitStream<UdpFramed<EternalReckoningCodec>>,
    tx: Tx,
}

impl Reader {
    pub fn new(
        shared: SharedState,
        stream: SplitStream<UdpFramed<EternalReckoningCodec>>,
        tx: Tx,
    ) -> Reader
    {
        Reader { shared, stream, tx }
    }

    fn receive(&mut self, addr: SocketAddr, op: Operation) -> Result<(), Error> {
        let mut shared = self.shared.lock()
            .map_err(|err| {
                format_err!("Failed to access shared state: {}", err)
            })?;

        if let Some(id) = shared.addr_to_id.get(&addr) {
            let id = *id;
            match op {
                Operation::DisconnectMessage => {
                    shared.addr_to_id.remove(&addr);
                    shared.id_to_addr.remove(&id);
                },
                _ => (),
            }
            self.tx.send((id, op))
                .map_err(|err| {
                    format_err!("Communication failure: {}", err)
                })?;
        } else {
            match op {
                Operation::ClConnectMessage(_) => {
                    let id = Uuid::new_v4();

                    shared.addr_to_id.insert(addr, id);
                    shared.id_to_addr.insert(id, addr);
                    
                    self.tx.send((id, op))
                        .map_err(|err| {
                            format_err!("Communication failure: {}", err)
                        })?;
                },
                _ => {
                    log::warn!("Received packet from unknown client: {}", &addr);
                },
            }
        }

        Ok(())
    }
}

impl Future for Reader {
    type Item = ();
    type Error = NetworkError;

    fn poll(&mut self) -> Poll<(), NetworkError> {
        loop {
            match self.stream.poll() {
                Ok(Async::NotReady) => {
                    return Ok(Async::NotReady);
                },
                Ok(Async::Ready(Some((op, addr)))) => {
                    self.receive(addr, op)
                        .map_err(|err| NetworkError::FatalError(
                            format_err!("Reader error: {}", err)
                        ))?;
                },
                Ok(Async::Ready(None)) => {
                    return Err(NetworkError::RebuildRequired);
                },
                Err(err) => {
                    return Err(NetworkError::FatalError(
                        format_err!("Reader error: {}", err)
                    ));
                }
            }
        }
    }
}