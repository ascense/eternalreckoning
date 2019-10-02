use std::sync::mpsc::{
    Sender,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use failure::{Error, format_err};
use futures::sync::mpsc;
use tokio::io;
use tokio::net::{
    TcpListener,
    TcpStream,
};
use tokio::prelude::*;
use tokio::codec::{FramedRead, FramedWrite};
use uuid::Uuid;

use eternalreckoning_core::net::{
    codec::EternalReckoningCodec,
    operation::{
        self,
        Operation,
    },
};
use crate::action::event::{
    self,
    ActionEvent,
    Update,
};

type Tx = mpsc::UnboundedSender<Operation>;
type Rx = mpsc::UnboundedReceiver<Operation>;

pub struct Server;

struct Shared {
    clients: HashMap<Uuid, Tx>,
}

impl Server {
    pub fn new() -> Server {
        Server {}
    }

    pub fn run(
        self,
        address: &String,
        update_rx: mpsc::UnboundedReceiver<Update>,
        sender: Sender<ActionEvent>
    )
    {
        let addr = address.parse().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();
        log::info!("Listening on: {}", addr);

        let state = Arc::new(Mutex::new(Shared::new()));

        let multiplex_state = state.clone();
        let multiplex = update_rx
            .for_each(move |update| {
                let op = match update.event {
                    event::UpdateEvent::MovementEvent(data) => {
                        Operation::ClMoveSetPosition(
                            operation::ClMoveSetPosition {
                                pos: data.position,
                            }
                        )
                    },
                };

                multiplex_state.lock().unwrap()
                    .clients.get(&update.uuid).unwrap()
                        .unbounded_send(op).map_err(|err| {
                            log::error!("Failed to send update to clients: {:?}", err);
                        });

                Ok(())
            });

        let server = listener.incoming()
            .for_each(move |socket| {
                process(socket, state.clone(), sender.clone());
                Ok(())
            })
            .map_err(|err| {
                println!("accept error = {:?}", err);
            });

        tokio::run(server.join(multiplex).map(|_| {}));
    }
}

impl Shared {
    pub fn new() -> Shared {
        Shared {
            clients: HashMap::new(),
        }
    }
}

fn process(socket: TcpStream, state: Arc<Mutex<Shared>>, sender: Sender<ActionEvent>) {
    log::debug!("New client socket established: {}", &socket.peer_addr().unwrap());

    let (reader, writer) = socket.split();

    let uuid = Uuid::new_v4();
    
    let framed_reader = FramedRead::new(reader, EternalReckoningCodec);
    let read_connection = ClientReader::new(framed_reader, uuid, state.clone(), sender)
        .map_err(|err| {
            eprintln!("client read failed: {:?}", err);
        });
    tokio::spawn(read_connection);

    let framed_writer = FramedWrite::new(writer, EternalReckoningCodec);
    let write_connection = ClientWriter::new(framed_writer, uuid, state)
        .map_err(|err| {
            eprintln!("client write failed: {:?}", err);
        });
    tokio::spawn(write_connection);
}

enum ClientReaderState {
    AwaitConnectionRequest,
    Connected,
}

struct ClientReader {
    uuid: Uuid,
    frames: FramedRead<io::ReadHalf<TcpStream>, EternalReckoningCodec>,
    event_tx: Sender<ActionEvent>,
    shared: Arc<Mutex<Shared>>,
    state: ClientReaderState,
}

impl ClientReader {
    fn new(
        frames: FramedRead<io::ReadHalf<TcpStream>, EternalReckoningCodec>,
        uuid: Uuid,
        shared: Arc<Mutex<Shared>>,
        event_tx: Sender<ActionEvent>
    ) -> ClientReader
    {
        ClientReader {
            uuid,
            frames,
            event_tx,
            shared,
            state: ClientReaderState::AwaitConnectionRequest,
        }
    }

    fn await_connection_request(&mut self, packet: &Operation) -> Result<(), Error> {
        match packet {
            Operation::ClConnectMessage(_) => {
                self.state = ClientReaderState::Connected;
                self.event_tx.send(ActionEvent::ConnectionEvent(
                    event::ConnectionEvent::ClientConnected(self.uuid)
                ))?;
                self.shared.lock().unwrap()
                    .clients.get(&self.uuid).unwrap()
                        .send(Operation::SvConnectResponse(
                            operation::SvConnectResponse {}
                        ));
                Ok(())
            },
            _ => Err(failure::format_err!(
                "unexpected client message: {}",
                packet
            )),
        }
    }

    fn connected(&mut self, packet: &Operation) -> Result<(), Error> {
        match packet {
            Operation::ClMoveSetPosition(data) => {
                self.event_tx.send(ActionEvent::MovementEvent(
                    crate::action::event::MovementEvent {
                        uuid: self.uuid,
                        position: data.pos,
                    }
                ));
                Ok(())
            },
            _ => Err(failure::format_err!(
                "unexpected client message: {}",
                packet
            )),
        }
    }
}

impl Drop for ClientReader {
    fn drop(&mut self) {
        match self.state {
            ClientReaderState::AwaitConnectionRequest => (),
            _ => {
                self.event_tx.send(ActionEvent::ConnectionEvent(
                    event::ConnectionEvent::ClientDisconnected(self.uuid)
                ));
            }
        };
    }
}

impl Future for ClientReader {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Error> {
        while let Async::Ready(frame) = self.frames.poll()? {
            if let Some(packet) = frame {
                match self.state {
                    ClientReaderState::AwaitConnectionRequest => self.await_connection_request(&packet)?,
                    ClientReaderState::Connected => self.connected(&packet)?,
                };
            } else {
                // EOF
                return Ok(Async::Ready(()));
            }
        }

        Ok(Async::NotReady)
    }
}

#[derive(PartialEq)]
enum ClientWriterState {
    Sending,
    Connected,
}

struct ClientWriter {
    uuid: Uuid,
    frames: FramedWrite<io::WriteHalf<TcpStream>, EternalReckoningCodec>,
    shared: Arc<Mutex<Shared>>,
    rx: Rx,
    state: ClientWriterState,
}

impl ClientWriter {
    pub fn new(
        frames: FramedWrite<io::WriteHalf<TcpStream>, EternalReckoningCodec>,
        uuid: Uuid,
        shared: Arc<Mutex<Shared>>,
    ) -> ClientWriter
    {
        let (tx, rx) = mpsc::unbounded();
        
        shared.lock().unwrap()
            .clients.insert(uuid, tx);

        ClientWriter {
            uuid,
            frames,
            shared,
            rx,
            state: ClientWriterState::Connected,
        }
    }
}

impl Future for ClientWriter {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Error> {
        loop {
            match self.state {
                ClientWriterState::Sending => {
                    futures::try_ready!(self.frames.poll_complete());
                    self.state = ClientWriterState::Connected;
                },
                ClientWriterState::Connected => {
                    match self.rx.poll().map_err(|_| format_err!("reader disconnected"))? {
                        Async::Ready(Some(op)) => {
                            self.frames.start_send(op)?;
                            self.state = ClientWriterState::Sending;
                        },
                        _ => break,
                    }
                }
            }
        }

        Ok(Async::NotReady)
    }
}

impl Drop for ClientWriter {
    fn drop(&mut self) {
        self.shared.lock().unwrap().clients
            .remove(&self.uuid);
    }
}