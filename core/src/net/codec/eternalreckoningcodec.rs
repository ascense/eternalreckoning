use bytes::BytesMut;
use failure::Error;
use lazy_static::lazy_static;
use tokio::codec::{Decoder, Encoder};

#[cfg(test)]
use bytes::{Buf, BufMut};

use crate::net::operation::Operation;

#[cfg(test)]
use crate::net::operation;

use super::{
    encdec,
    error::CodecError,
    header::{
        Header,
        HEADER_SIZE,
    },
    opcode::{
        self,
        opcode_from_operation,
    },
};

type EncoderFn = fn(Operation, &mut BytesMut);
type DecoderFn = fn(&Header, &mut BytesMut) -> Result<Option<Operation>, CodecError>;
lazy_static! {
    static ref ENCODER_TABLE: [EncoderFn; std::u8::MAX as usize + 1] = {
        let mut table = [encdec::encode_no_body as EncoderFn; std::u8::MAX as usize + 1];
        table[opcode::SV_CONNECT_RESPONSE_OP as usize] = encdec::encode_sv_connect_response;
        table[opcode::SV_UPDATE_WORLD_OP as usize] = encdec::encode_sv_update_world;
        table[opcode::CL_MOVE_SET_POSITION_OP as usize] = encdec::encode_cl_move_set_position;
        table
    };

    static ref DECODER_TABLE: [DecoderFn; std::u8::MAX as usize + 1] = {
        let mut table = [encdec::decode_invalid_op as DecoderFn; std::u8::MAX as usize + 1];
        table[opcode::CL_CONNECT_MESSAGE_OP as usize] = encdec::decode_cl_connect_message;
        table[opcode::SV_CONNECT_RESPONSE_OP as usize] = encdec::decode_sv_connect_response;
        table[opcode::SV_UPDATE_WORLD_OP as usize] = encdec::decode_sv_update_world;
        table[opcode::DISCONNECT_MESSAGE_OP as usize] = encdec::decode_disconnect_message;
        table[opcode::CL_MOVE_SET_POSITION_OP as usize] = encdec::decode_cl_move_set_position;
        table
    };
}

pub struct EternalReckoningCodec;

impl Encoder for EternalReckoningCodec {
    type Item = Operation;
    type Error = Error;

    fn encode(&mut self, packet: Self::Item, buf: &mut BytesMut)
        -> Result<(), Self::Error>
    {
        log::trace!("Encoding: {}", &packet);

        buf.reserve(HEADER_SIZE);
        let mut payload_buf = buf.split_off(HEADER_SIZE);

        let opcode = opcode_from_operation(&packet);
        ENCODER_TABLE[opcode as usize](packet, &mut payload_buf);
        Header::new(opcode, payload_buf.len()).write(buf);

        buf.unsplit(payload_buf);

        Ok(())
    }
}

impl Decoder for EternalReckoningCodec {
    type Item = Operation;
    type Error = Error;

    fn decode(&mut self, buf: &mut BytesMut)
        -> Result<Option<Self::Item>, Self::Error>
    {
        if buf.len() == 0 {
            return Ok(Some(Operation::DisconnectMessage));
        }

        loop {
            if let Some(packet_pos) = Header::find(buf) {
                buf.advance(packet_pos);
            } else {
                return Ok(None);
            }

            match Header::read(buf) {
                Ok(None) => return Ok(None),
                Err(_) => (),
                Ok(Some(header)) => {
                    let mut payload_buf = buf.split_off(HEADER_SIZE);
                    let result = DECODER_TABLE[header.opcode as usize](&header, &mut payload_buf);
                    buf.unsplit(payload_buf);
                    match result {
                        Ok(None) => return Ok(None),
                        Ok(Some(op)) => {
                            buf.split_to(HEADER_SIZE + header.size);
                            return Ok(Some(op));
                        },
                        Err(err) => {
                            log::debug!("{}", err);
                        },
                    }
                },
            }

            buf.advance(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_header() {
        let mut codec = EternalReckoningCodec;
        let mut buf = BytesMut::with_capacity(8);

        let op = Operation::ClConnectMessage(operation::ClConnectMessage);

        codec.encode(op, &mut buf).unwrap();

        let mut cursor = std::io::Cursor::new(&buf);

        // magic
        assert_eq!(cursor.get_u8(), 0xEC);
        assert_eq!(cursor.get_u8(), 0xAA);

        // size
        assert_eq!(cursor.get_u16_le(), 0);

        // opcode
        assert_eq!(cursor.get_u8(), opcode::CL_CONNECT_MESSAGE_OP);

        // padding
        assert_eq!(cursor.remaining(), 3);
    }

    #[test]
    fn test_decode_empty_buffer() {
        let mut codec = EternalReckoningCodec;
        let mut buf = BytesMut::from(&[][..]);

        match codec.decode(&mut buf) {
            Ok(Some(Operation::DisconnectMessage)) => (),
            _ => panic!("Invalid decode for EOF"),
        }
    }

    #[test]
    fn test_decode_header() {
        let mut codec = EternalReckoningCodec;
        let mut buf = BytesMut::with_capacity(8);

        // magic
        buf.put_slice(&[0xEC, 0xAA][..]);

        // size
        buf.put_u16_le(0);

        // opcode
        buf.put_u8(opcode::CL_CONNECT_MESSAGE_OP);

        // padding
        buf.put_slice(&[0x00, 0x00, 0x00][..]);

        match codec.decode(&mut buf) {
            Ok(Some(Operation::ClConnectMessage(operation::ClConnectMessage))) => (),
            _ => panic!("Invalid decode for ClConnectMessage"),
        }
    }

    #[test]
    fn test_decode_empty_world_update() {
        let mut codec = EternalReckoningCodec;
        let mut buf = BytesMut::with_capacity(8 + 24);

        let uuid = uuid::Uuid::from_slice(
            &b"\xd1qHq\xdb\xbdNe\xa9f\xc6\xe5|I\xbaG"[..]
        ).unwrap();
        
        // header
        buf.put_slice(&b"\xec\xaa"[..]);
        buf.put_u16_le(24);
        buf.put_u8(opcode::SV_UPDATE_WORLD_OP);
        buf.put_slice(&b"\0\0\0"[..]);

        // entity count
        buf.put_u32_le(1);

        // UUID
        for byte in uuid.as_bytes() {
            buf.put_u8(*byte);
        }

        // component count
        buf.put_u32_le(0);
        
        match codec.decode(&mut buf) {
            Ok(Some(Operation::SvUpdateWorld(data))) => {
                assert_eq!(data.updates.len(), 1);
                let update = data.updates.get(0).unwrap();

                assert_eq!(&update.uuid, &uuid);
                assert_eq!(update.data.len(), 0);
            },
            _ => panic!("Invalid decode for SvUpdateWorld"),
        }
    }
}