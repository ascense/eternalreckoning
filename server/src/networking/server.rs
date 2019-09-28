use std::sync::mpsc::{
    Receiver,
    Sender,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use failure::Error;
use futures::future::Either;
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
    packet::{
        Packet,
        Operation,
    },
};
use crate::action::event::{
    ActionEvent,
    ConnectionEvent,
    Update,
};

type Tx = mpsc::UnboundedSender<Bytes>;
type Rx = mpsc::UnboundedReceiver<Bytes>;

pub struct Server;

struct Shared {
    clients: HashMap<Uuid, Tx>,
}

impl Server {
    pub fn new() -> Server {
        Server {}
    }

    pub fn run(self, address: &String, sender: Sender<ActionEvent>) {
        let addr = address.parse().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();
        log::info!("Listening on: {}", addr);

        let state = Arc::new(Mutex::new(Shared::new()));

        let server = listener.incoming()
            .for_each(move |socket| {
                process(socket, state.clone(), sender.clone());
                Ok(())
            })
            .map_err(|err| {
                println!("accept error = {:?}", err);
            });
        
        tokio::run(server);
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

    fn await_connection_request(&mut self, packet: &Packet) -> Result<(), Error> {
        match packet.operation {
            Operation::ConnectReq => {
                self.state = ClientReaderState::Connected;
                self.event_tx.send(ActionEvent::ConnectionEvent(
                    ConnectionEvent::ClientConnected(self.uuid)
                ))?;
                self.shared.lock().unwrap()
                    .clients.get(&self.uuid).unwrap()
                        .unbounded_send(Bytes::from("a"))?;
                Ok(())
            },
            _ => Err(failure::format_err!(
                "unexpected client message: {}",
                packet.operation
            )),
        }
    }

    fn connected(&self, packet: &Packet) -> Result<(), Error> {
        Ok(())
    }
}

impl Drop for ClientReader {
    fn drop(&mut self) {
        match self.state {
            ClientReaderState::AwaitConnectionRequest => (),
            _ => {
                self.event_tx.send(ActionEvent::ConnectionEvent(
                    ConnectionEvent::ClientDisconnected(self.uuid)
                ));
            }
        };
    }
}

impl Future for ClientReader {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Error> {
        log::trace!("reader poll");

        while let Async::Ready(frame) = self.frames.poll()? {
            if let Some(packet) = frame {
                log::trace!("reader packet");
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
                    match self.rx.poll().unwrap() {
                        Async::Ready(Some(_)) => {
                            self.frames.start_send(Packet {
                                operation: Operation::ConnectRes,
                            });
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