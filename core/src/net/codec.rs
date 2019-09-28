use bytes::{BufMut, BytesMut};
use tokio::codec::{Decoder, Encoder};
use failure::Error;
use failure_derive::Fail;

use super::packet;

type OpcodeType = u8;

#[derive(Debug, Fail)]
pub enum CodecError {
    #[fail(display = "invalid opcode: {}", _0)]
    InvalidOpcode(OpcodeType),
}

pub struct EternalReckoningCodec;

impl Encoder for EternalReckoningCodec {
    type Item = packet::Packet;
    type Error = Error;

    fn encode(&mut self, packet: Self::Item, buf: &mut BytesMut)
        -> Result<(), Self::Error>
    {
        log::trace!("Out: {}", &packet.operation);

        buf.reserve(std::mem::size_of::<OpcodeType>());
        buf.put(packet.operation as OpcodeType);

        Ok(())
    }
}

impl Decoder for EternalReckoningCodec {
    type Item = packet::Packet;
    type Error = Error;

    fn decode(&mut self, buf: &mut BytesMut)
        -> Result<Option<Self::Item>, Self::Error>
    {
        let data = buf.split_to(1);
        if data.len() <= 0 {
            return Ok(None);
        }
        let data = data[0];

        log::trace!("In: {:?}", data);
        if let Some(operation) = packet::Operation::from_value(data) {
            Ok(Some(packet::Packet { operation }))
        } else {
            Err(CodecError::InvalidOpcode(data).into())
        }
    }
}