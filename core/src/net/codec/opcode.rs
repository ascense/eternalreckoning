use crate::net::operation::Operation;

pub type OpcodeType = u8;

pub const CL_SYNC_OP: OpcodeType = 0x00;
pub const SV_SYNC_OP: OpcodeType = 0x01;
pub const CL_CONNECT_MESSAGE_OP: OpcodeType = 0x02;
pub const SV_CONNECT_RESPONSE_OP: OpcodeType = 0x03;
pub const SV_UPDATE_WORLD_OP: OpcodeType = 0x10;
pub const CL_MOVE_SET_POSITION_OP: OpcodeType = 0x20;
pub const DISCONNECT_MESSAGE_OP: OpcodeType = 0xFF;

pub fn opcode_from_operation(op: &Operation) -> OpcodeType {
    match op {
        Operation::ClSync(_) => CL_SYNC_OP,
        Operation::SvSync(_) => SV_SYNC_OP,
        Operation::ClConnectMessage(_) => CL_CONNECT_MESSAGE_OP,
        Operation::SvConnectResponse(_) => SV_CONNECT_RESPONSE_OP,
        Operation::SvUpdateWorld(_) => SV_UPDATE_WORLD_OP,
        Operation::ClMoveSetPosition(_) => CL_MOVE_SET_POSITION_OP,
        Operation::DisconnectMessage => DISCONNECT_MESSAGE_OP,
    }
}