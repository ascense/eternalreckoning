use tokio::net::{
    UdpSocket,
    UdpFramed,
};
use tokio::prelude::*;

use eternalreckoning_core::net::{
    codec::EternalReckoningCodec,
    operation::{
        self,
        Operation,
    },
};

fn main() {
    let addr = ([127, 0, 0, 1], 6142).into();

    let socket = UdpSocket::bind(&([127, 0, 0, 1], 0).into()).unwrap();
    socket.connect(&addr).unwrap();

    let stream = UdpFramed::new(
        socket,
        EternalReckoningCodec
    );

    let sequence = stream
        .send((Operation::ClConnectMessage(operation::ClConnectMessage), addr))
        .and_then(|framed| {
            framed.into_future().map_err(|(err, _stream)| err)
        })
        .map(|(op, _stream)| {
            assert!(op.is_some());

            let op = op.unwrap();

            if let Operation::SvConnectResponse(operation::SvConnectResponse { uuid }) = op.0 {
                println!("Connected with UUID {}", uuid);
                eprintln!("Result: OK");
            } else {
                panic!("Invalid response from server");
            }
        })
        .map_err(|err| {
            panic!(err);
        });

    tokio::run(sequence);
}