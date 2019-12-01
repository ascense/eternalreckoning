use failure_derive::Fail;

use super::opcode::OpcodeType;

#[derive(Debug, Fail)]
pub enum CodecError {
    #[fail(display = "invalid opcode: {:02X}", _0)]
    InvalidOpcode(OpcodeType),
    #[fail(display = "invalid data")]
    BadData,
}