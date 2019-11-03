use failure::{
    format_err,
    Error,
};
use futures::stream::{
    Stream,
    SplitSink,
};
use futures::sink::Sink;
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

pub type Rx = futures::sync::mpsc::UnboundedReceiver<(Uuid, Operation)>;

pub struct Writer {
    shared: SharedState,
    sink: SplitSink<UdpFramed<EternalReckoningCodec>>,
    rx: Rx,
    state: WriterState,
}

#[derive(PartialEq)]
enum WriterState {
    Idle,
    Sending,
}

impl Writer {
    pub fn new(
        shared: SharedState,
        sink: SplitSink<UdpFramed<EternalReckoningCodec>>,
        rx: Rx,
    ) -> Writer
    {
        let state = WriterState::Idle;

        Writer { shared, sink, rx, state }
    }

    fn send(&mut self, client: Uuid, op: Operation) -> Result<(), Error> {
        let shared = self.shared.lock()
            .map_err(|err| {
                format_err!("Failed to access shared state: {}", err)
            })?;

        if let Some(addr) = shared.id_to_addr.get(&client) {
            self.sink.start_send((op, *addr))?;
        } else {
            log::warn!("Attempted to send to unknown client {}", client);
        }

        Ok(())
    }

    fn poll_sending(&mut self) -> Poll<(), Error> {
        futures::try_ready!(self.sink.poll_complete());

        Ok(Async::Ready(()))
    }

    fn poll_idle(&mut self) -> Poll<(), Error> {
        match self.rx.poll().map_err(|_| format_err!("Reader disconnected"))? {
            Async::Ready(Some((client, op))) => {
                self.send(client, op)?;
                return Ok(Async::Ready(()));
            },
            Async::NotReady => return Ok(Async::NotReady),
            Async::Ready(None) => {
                return Err(format_err!("Reader disconnected"));
            },
        }
    }
}

impl Future for Writer {
    type Item = ();
    type Error = NetworkError;

    fn poll(&mut self) -> Poll<(), NetworkError> {
        loop {
            match self.state {
                WriterState::Sending => {
                    futures::try_ready!(
                        self.poll_sending()
                            .map_err(|err| NetworkError::FatalError(
                                format_err!("Writer error: {}", err)
                            ))
                    );
                    self.state = WriterState::Idle;
                },
                WriterState::Idle => {
                    futures::try_ready!(
                        self.poll_idle()
                            .map_err(|err| NetworkError::FatalError(
                                format_err!("Writer error: {}", err)
                            ))
                    );
                    self.state = WriterState::Sending;
                },
            }
        }
    }
}