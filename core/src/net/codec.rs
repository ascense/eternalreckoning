use bytes::{Buf, BufMut, BytesMut};
use tokio::codec::{Decoder, Encoder};
use failure::Error;
use failure_derive::Fail;

use super::operation::{
    self,
    Operation,
};

#[derive(Debug, Fail)]
pub enum CodecError {
    #[fail(display = "invalid opcode: {}", _0)]
    InvalidOpcode(OpcodeType),
    #[fail(display = "invalid data")]
    BadData,
}

type OpcodeType = u8;
const OPCODE_LEN: usize = std::mem::size_of::<OpcodeType>();

const CL_CONNECT_MESSAGE_OP: OpcodeType = 0x01;
const SV_CONNECT_RESPONSE_OP: OpcodeType = 0x02;
const SV_UPDATE_WORLD_OP: OpcodeType = 0x10;
const CL_MOVE_SET_POSITION_OP: OpcodeType = 0x20;

pub struct EternalReckoningCodec;

impl Encoder for EternalReckoningCodec {
    type Item = Operation;
    type Error = Error;

    fn encode(&mut self, packet: Self::Item, buf: &mut BytesMut)
        -> Result<(), Self::Error>
    {
        log::trace!("Encoding: {}", &packet);

        buf.reserve(std::mem::size_of::<OpcodeType>());
        match packet {
            Operation::ClConnectMessage(_) => {    
                buf.put(CL_CONNECT_MESSAGE_OP);
                Ok(())
            },
            Operation::SvConnectResponse(_) => {
                buf.put(SV_CONNECT_RESPONSE_OP);
                Ok(())
            },
            Operation::SvUpdateWorld(data) => {
                buf.put(SV_UPDATE_WORLD_OP);
                self.encode_svupdateworld(data, buf)
            },
            Operation::ClMoveSetPosition(data) => {
                buf.put(CL_MOVE_SET_POSITION_OP);
                self.encode_clmovesetposition(data, buf)
            },
        }
    }
}

impl Decoder for EternalReckoningCodec {
    type Item = Operation;
    type Error = Error;

    fn decode(&mut self, buf: &mut BytesMut)
        -> Result<Option<Self::Item>, Self::Error>
    {
        let opcode = match buf.get(0) {
            Some(opcode) => opcode,
            None => return Ok(None),
        };

        let packet = match *opcode {
            CL_CONNECT_MESSAGE_OP => self.decode_clconnectmessage(buf)?,
            SV_CONNECT_RESPONSE_OP => self.decode_svconnectresponse(buf)?,
            SV_UPDATE_WORLD_OP => self.decode_svupdateworld(buf)?,
            CL_MOVE_SET_POSITION_OP => self.decode_clmovesetposition(buf)?,
            _ => return Err(CodecError::InvalidOpcode(*opcode).into()),
        };
        match packet {
            Some(ref packet) => log::trace!("Decoded: {}", packet),
            None => (),
        }
        
        Ok(packet)
    }
}

impl EternalReckoningCodec {
    fn encode_svupdateworld(
        &self,
        data: operation::SvUpdateWorld,
        buf: &mut BytesMut,
    ) -> Result<(), Error>
    {
        Ok(())
    }
    
    fn encode_clmovesetposition(
        &self,
        data: operation::ClMoveSetPosition,
        buf: &mut BytesMut,
    ) -> Result<(), Error>
    {
        let coords = &data.pos.coords;
        buf.reserve(3*std::mem::size_of::<f64>());
        buf.put_f64_le(coords.x);
        buf.put_f64_le(coords.y);
        buf.put_f64_le(coords.z);
        Ok(())
    }

    fn decode_clconnectmessage(
        &self,
        buf: &mut BytesMut,
    ) -> Result<Option<Operation>, Error>
    {
        buf.split_to(OPCODE_LEN);
        Ok(Some(Operation::ClConnectMessage(operation::ClConnectMessage)))
    }

    fn decode_svconnectresponse(
        &self,
        buf: &mut BytesMut,
    ) -> Result<Option<Operation>, Error>
    {
        buf.split_to(OPCODE_LEN);
        Ok(Some(Operation::SvConnectResponse(operation::SvConnectResponse)))
    }

    fn decode_svupdateworld(
        &self,
        buf: &mut BytesMut,
    ) -> Result<Option<Operation>, Error>
    {
        buf.split_to(OPCODE_LEN);
        Ok(Some(Operation::SvUpdateWorld(
            operation::SvUpdateWorld {
                updates: Vec::new(),
            }
        )))
    }

    fn decode_clmovesetposition(
        &self,
        buf: &mut BytesMut,
    ) -> Result<Option<Operation>, Error>
    {
        let data_len = 3*std::mem::size_of::<f64>();
        let data = match buf.get(OPCODE_LEN..data_len+1) {
            Some(data) => data,
            None => return Ok(None),
        };

        let mut data = std::io::Cursor::new(data);
        let packet = operation::ClMoveSetPosition {
            pos: nalgebra::Point3::<f64>::new(
                data.get_f64_le(),
                data.get_f64_le(),
                data.get_f64_le(),
            ),
        };

        buf.split_to(OPCODE_LEN + data_len);
        Ok(Some(Operation::ClMoveSetPosition(packet)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_empty_buffer() {
        let mut codec = EternalReckoningCodec;
        let mut buf = BytesMut::from(&[][..]);

        let packet = codec.decode(&mut buf);
        assert!(&packet.is_ok());
        assert!(&packet.unwrap().is_none());
    }

    #[test]
    fn test_decode_client_connect_message() {
        let mut codec = EternalReckoningCodec;
        let mut buf = BytesMut::from(&[CL_CONNECT_MESSAGE_OP][..]);

        let packet = codec.decode(&mut buf);

        assert!(&packet.is_ok());
        let packet = packet.unwrap();
        assert!(&packet.is_some());

        match packet.unwrap() {
            Operation::ClConnectMessage(_) => (),
            _ => panic!("Operation != ClConnectMessage"),
        };
    }

    #[test]
    fn test_decode_client_move_set_position() {
        let mut codec = EternalReckoningCodec;
        let mut buf = BytesMut::with_capacity(1 + 3*8);

        buf.put_u8(CL_MOVE_SET_POSITION_OP);
        buf.put_f64_le(0.0);
        buf.put_f64_le(1.0);
        buf.put_f64_le(2.0);

        let mut buf = BytesMut::from(buf);
        let packet = codec.decode(&mut buf);

        assert!(&packet.is_ok());
        let packet = packet.unwrap();
        assert!(&packet.is_some());

        match packet.unwrap() {
            Operation::ClMoveSetPosition(data) => {
                assert_eq!(data.pos.x, 0.0);
                assert_eq!(data.pos.y, 1.0);
                assert_eq!(data.pos.z, 2.0);
            },
            _ => panic!("Operation != ClConnectMessage"),
        };
    }
}