use bytes::{Buf, BufMut, BytesMut};
use failure::Error;
use failure::format_err;
use failure_derive::Fail;
use tokio::codec::{Decoder, Encoder};
use uuid::Uuid;

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

type ComponentCodeType = u8;
const COMPONENT_CODE_LEN: usize = std::mem::size_of::<ComponentCodeType>();

const HEALTH_COMP: ComponentCodeType = 0x01;
const POSITION_COMP: ComponentCodeType = 0x02;

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
            _ => {
                return Err(CodecError::InvalidOpcode(*opcode).into());
            },
        };
        match packet {
            Some(ref packet) => log::trace!("Decoded: {}", packet),
            None => (),
        }
        
        Ok(packet)
    }
}

impl EternalReckoningCodec {
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

    fn encode_svupdateworld(
        &self,
        data: operation::SvUpdateWorld,
        buf: &mut BytesMut,
    ) -> Result<(), Error>
    {
        buf.reserve(4);
        buf.put_u32_le(data.updates.len() as u32);

        for entity in &data.updates {
            buf.reserve(16);
            for byte in entity.uuid.as_bytes() {
                buf.put_u8(*byte);
            }

            buf.reserve(4);
            buf.put_u32_le(entity.data.len() as u32);

            for component in &entity.data {
                self.encode_entity_component(&component, buf);
            }
        }

        Ok(())
    }

    fn decode_svupdateworld(
        &self,
        buf: &mut BytesMut,
    ) -> Result<Option<Operation>, Error>
    {
        let mut data = std::io::Cursor::new(&buf);

        let mut read_size = OPCODE_LEN;
        data.advance(OPCODE_LEN);

        if data.remaining() < 4 {
            return Ok(None);
        }
        read_size += 4;
        let data_count = data.get_u32_le();

        let mut updates = Vec::new();
        for _ in 0..data_count {
            // UUID + component count
            if data.remaining() < 16+4 {
                return Ok(None);
            }
            read_size += 16+4;

            let mut uuid_buf: [u8; 16] = [0; 16];
            data.copy_to_slice(&mut uuid_buf);
            let uuid = Uuid::from_slice(&uuid_buf[..])?;

            let component_count = data.get_u32_le();

            let mut component_data = Vec::new();
            for _ in 0..component_count {
                match self.decode_entity_component(&mut data) {
                    Ok(Some((size, component))) => {
                        read_size += size;
                        component_data.push(component);
                    },
                    Ok(None) => return Ok(None),
                    Err(err) => return Err(err),
                };
            }

            updates.push(operation::EntityUpdate { uuid, data: component_data });
        }

        buf.split_to(read_size);
        Ok(Some(Operation::SvUpdateWorld(
            operation::SvUpdateWorld { updates }
        )))
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

    fn encode_entity_component(
        &self,
        data: &operation::EntityComponent,
        buf: &mut BytesMut,
    )
    {
        match data {
            operation::EntityComponent::Health(health) => {
                buf.reserve(COMPONENT_CODE_LEN + 8);
                buf.put_u8(HEALTH_COMP);
                buf.put_u64_le(*health);
            },
            operation::EntityComponent::Position(position) => {
                buf.reserve(COMPONENT_CODE_LEN + 24);
                buf.put_u8(POSITION_COMP);
                buf.put_f64_le(position.x);
                buf.put_f64_le(position.y);
                buf.put_f64_le(position.z);
            },
        }
    }

    fn decode_entity_component(
        &self,
        data: &mut std::io::Cursor<&&mut BytesMut>,
    ) -> Result<Option<(usize, operation::EntityComponent)>, Error>
    {
        if data.remaining() < COMPONENT_CODE_LEN {
            return Ok(None);
        }
        let component_code = data.get_u8();

        match component_code {
            HEALTH_COMP => {
                if data.remaining() < 8 {
                    Ok(None)
                } else {
                    Ok(Some((
                        COMPONENT_CODE_LEN + 8,
                        operation::EntityComponent::Health(
                            data.get_u64_le()
                        )
                    )))
                }
            },
            POSITION_COMP => {
                if data.remaining() < 24 {
                    Ok(None)
                } else {
                    Ok(Some((
                        COMPONENT_CODE_LEN + 24,
                        operation::EntityComponent::Position(
                            nalgebra::Point3::<f64>::new(
                                data.get_f64_le(),
                                data.get_f64_le(),
                                data.get_f64_le(),
                            )
                        )
                    )))
                }
            },
            _ => Err(format_err!("unknown component update code: {}", component_code)),
        }
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

    #[test]
    fn test_encode_and_decode_world_update() {
        let mut codec = EternalReckoningCodec;
        let len =
            1 + // opcode
            4 + // update count
            16 + // UUID
            4 + // component count
            1 + // component code
            24; // position
        let buf = BytesMut::with_capacity(len);

        let mut buf = BytesMut::from(buf);
        let uuid = Uuid::new_v4();
        let position = nalgebra::Point3::<f64>::new(0.0, 1.0, 2.0);
        let packet = Operation::SvUpdateWorld(
            operation::SvUpdateWorld {
                updates: vec![operation::EntityUpdate {
                    uuid,
                    data: vec![operation::EntityComponent::Position(position)],
                }],
            }
        );

        codec.encode(packet.clone(), &mut buf).unwrap();
        assert_eq!(buf.len(), buf.capacity());

        let decoded = codec.decode(&mut buf);
        assert!(decoded.is_ok());
        let decoded = decoded.unwrap();
        assert!(decoded.is_some());

        if let Operation::SvUpdateWorld(decoded_op) = decoded.unwrap() {
            assert_eq!(decoded_op.updates.len(), 1);

            let update = decoded_op.updates.get(0).unwrap();
            assert_eq!(update.uuid, uuid);
            assert_eq!(update.data.len(), 1);

            if let operation::EntityComponent::Position(pos) = update.data.get(0).unwrap() {
                assert_eq!(pos, &position);
            } else {
                panic!("decoded as incorrect component update");
            }
        } else {
            panic!("decoded as incorrect operation");
        }

        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_decode_incomplete_world_update() {
        let mut codec = EternalReckoningCodec;

        let mut buf = BytesMut::with_capacity(41);
        buf.put_u8(0x10);
        buf.put_u32_le(1);
        buf.put_slice(&b"\x13\xfc\x8c\x89yjBG\x9c\xfc\xf4\ng\x84\x19)"[..]);
        buf.put_u32_le(1);
        buf.put_u8(0x02);
        buf.put_u64_le(0);
        buf.put_slice(&b"\0\0\0\0\0\0\0"[..]);

        match codec.decode(&mut buf) {
            Ok(Some(_)) => panic!("decoded packet from an incomplete buffer"),
            Ok(None) => (),
            Err(_) => panic!("decoding error from a valid stream"),
        }
    }
}