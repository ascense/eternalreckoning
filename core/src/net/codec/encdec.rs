use bytes::{Buf, BufMut, BytesMut};
use uuid::Uuid;

use crate::net::operation::{
    self,
    Operation,
};

use super::{
    error::CodecError,
    header::Header,
};

type ComponentCodeType = u8;
const COMPONENT_CODE_LEN: usize = std::mem::size_of::<ComponentCodeType>();

const HEALTH_COMP: ComponentCodeType = 0x01;
const POSITION_COMP: ComponentCodeType = 0x02;

// FIXME: review decoders & handle incomplete data

pub fn encode_no_body(_op: Operation, _buf: &mut BytesMut) {}

pub fn decode_invalid_op(header: &Header, _buf: &mut BytesMut)
    -> Result<Option<Operation>, CodecError>
{
    Err(CodecError::InvalidOpcode(header.opcode))
}

pub fn decode_cl_connect_message(header: &Header, _buf: &mut BytesMut)
    -> Result<Option<Operation>, CodecError>
{
    if header.size != 0 {
        return Err(CodecError::BadData);
    }

    Ok(Some(Operation::ClConnectMessage(operation::ClConnectMessage)))
}

pub fn encode_sv_connect_response(op: Operation, buf: &mut BytesMut) {
    if let Operation::SvConnectResponse(data) = op {
        for byte in data.uuid.as_bytes() {
            buf.put_u8(*byte);
        }
    } else {
        panic!("Invalid encoder function called!");
    }
}

pub fn decode_sv_connect_response(header: &Header, buf: &mut BytesMut)
    -> Result<Option<Operation>, CodecError>
{
    if header.size != 16 {
        return Err(CodecError::BadData);
    }
    if buf.len() < header.size {
        return Ok(None);
    }

    let uuid = Uuid::from_slice(&buf[0..16])
        .map_err(|_| CodecError::BadData)?;

    Ok(Some(Operation::SvConnectResponse(
        operation::SvConnectResponse { uuid }
    )))
}

pub fn encode_sv_update_world(op: Operation, buf: &mut BytesMut) {
    if let Operation::SvUpdateWorld(data) = op {
        buf.reserve(4 + data.updates.len() * (16+4));
        buf.put_u32_le(data.updates.len() as u32);

        for entity in &data.updates {
            for byte in entity.uuid.as_bytes() {
                buf.put_u8(*byte);
            }

            buf.put_u32_le(entity.data.len() as u32);

            for component in &entity.data {
                encode_entity_component(&component, buf);
            }
        }
    } else {
        panic!("Invalid encoder function called!");
    }
}

fn encode_entity_component(
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

pub fn decode_sv_update_world(header: &Header, buf: &mut BytesMut)
    -> Result<Option<Operation>, CodecError>
{
    if header.size < 4 {
        return Err(CodecError::BadData);
    }

    let mut data = std::io::Cursor::new(&buf);

    if data.remaining() < 4 {
        return Ok(None);
    }
    let data_count = data.get_u32_le();

    let mut updates = Vec::new();
    for _ in 0..data_count {
        // UUID + component count
        if data.remaining() < 16+4 {
            return Ok(None);
        }

        let mut uuid_buf: [u8; 16] = [0; 16];
        data.copy_to_slice(&mut uuid_buf);
        let uuid = Uuid::from_slice(&uuid_buf[..]).unwrap();

        let component_count = data.get_u32_le();

        let mut component_data = Vec::new();
        for _ in 0..component_count {
            match decode_entity_component(&mut data) {
                Ok(Some((size, component))) => {
                    component_data.push(component);
                },
                Ok(None) => return Ok(None),
                Err(err) => return Err(err),
            };
        }

        updates.push(operation::EntityUpdate { uuid, data: component_data });
    }

    Ok(Some(Operation::SvUpdateWorld(
        operation::SvUpdateWorld { updates }
    )))
}

fn decode_entity_component(
    data: &mut std::io::Cursor<&&mut BytesMut>,
) -> Result<Option<(usize, operation::EntityComponent)>, CodecError>
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
        _ => Err(CodecError::BadData),
    }
}

pub fn encode_cl_move_set_position(op: Operation, buf: &mut BytesMut) {
    if let Operation::ClMoveSetPosition(data) = op {
        buf.reserve(3 * std::mem::size_of::<f64>());
        
        let coords = &data.pos.coords;
        buf.put_f64_le(coords.x);
        buf.put_f64_le(coords.y);
        buf.put_f64_le(coords.z);
    } else {
        panic!("Invalid encoder function called!");
    }
}

pub fn decode_cl_move_set_position(header: &Header, buf: &mut BytesMut)
    -> Result<Option<Operation>, CodecError>
{
    if header.size != 3 * std::mem::size_of::<f64>() {
        return Err(CodecError::BadData);
    }
    if buf.len() < header.size {
        return Ok(None);
    }

    let mut data = std::io::Cursor::new(buf);
    Ok(Some(Operation::ClMoveSetPosition(
        operation::ClMoveSetPosition {
            pos: nalgebra::Point3::<f64>::new(
                data.get_f64_le(),
                data.get_f64_le(),
                data.get_f64_le(),
            ),
        }
    )))
}

pub fn decode_disconnect_message(_header: &Header, _buf: &mut BytesMut)
    -> Result<Option<Operation>, CodecError>
{
    Ok(Some(Operation::DisconnectMessage))
}