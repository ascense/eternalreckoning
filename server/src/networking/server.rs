use std::sync::mpsc::Sender;

use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::codec::Decoder;
use tokio_codec::BytesCodec;
use uuid::Uuid;

use crate::action::{
    ActionEvent,
    ConnectionEvent,
};

pub struct Server;

impl Server {
    pub fn new() -> Server {
        Server {}
    }

    pub fn run(self, sender: Sender<ActionEvent>) {
        let addr = "127.0.0.1:6142".parse().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();
        println!("Listening on: {}", addr);

        let server = listener.incoming()
            .for_each(move |socket| {
                let framed = BytesCodec::new().framed(socket);
                let (_writer, reader) = framed.split();

                let client_id = Uuid::new_v4();
                sender.send(ActionEvent::ConnectionEvent(
                    ConnectionEvent::ClientConnected(client_id)
                ));

                let thread_sender = sender.clone();

                let processor = reader
                    .for_each(|bytes| {
                        println!("bytes: {:?}", bytes);
                        Ok(())
                    })
                    .or_else(|err| {
                        println!("Socket closed with error: {:?}", err);
                        Err(err)
                    })
                    .then(move |result| {
                        println!("Socket closed with result: {:?}", result);
                        thread_sender.send(ActionEvent::ConnectionEvent(
                            ConnectionEvent::ClientDisconnected(client_id)
                        ));
                        Ok(())
                    });

                tokio::spawn(processor);
                Ok(())
            })
            .map_err(|err| {
                println!("accept error = {:?}", err);
            });
        
        tokio::run(server);
    }
}

/*
struct Client {
    uuid: Uuid,
    event_tx: Sender<ActionEvent>,
    rx: Rx,
    addr: SocketAddr,
};

impl Client {
    fn new(socket: TcpStream, event_tx: Sender<ActionEvent>) -> Client {
        let uuid = Uuid::v4();

        Client { uuid, event_tx }
    }
}
*/