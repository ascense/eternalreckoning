mod encdec;
mod error;
mod eternalreckoningcodec;
mod header;
mod opcode;

pub use self::{
    error::CodecError,
    eternalreckoningcodec::EternalReckoningCodec,
};