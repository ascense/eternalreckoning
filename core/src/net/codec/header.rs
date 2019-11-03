use std::io::{
    Cursor,
    Seek,
    SeekFrom,
};

use bytes::{Buf, BufMut, BytesMut};

use super::error::CodecError;
use super::opcode::OpcodeType;

type MagicType = [u8; 2];
const PACKET_MAGIC: MagicType = [0xEC, 0xAA];

pub static HEADER_SIZE: usize = 8;
pub struct Header {
    pub size: usize,
    pub opcode: OpcodeType,
}

impl Header {
    pub fn new(opcode: OpcodeType, size: usize) -> Header {
        Header { size, opcode }
    }

    pub fn find(buf: &mut BytesMut) -> Option<usize> {
        let mut data = Cursor::new(&buf);
        while data.remaining() >= PACKET_MAGIC.len() {
            let position = data.position();

            for byte in &PACKET_MAGIC[..] {
                if data.get_u8() != *byte {
                    data.seek(SeekFrom::Start(position + 1))
                        .unwrap(); // safe, as we just were there
                    break;
                }
            }

            return Some(position as usize);
        }

        None
    }

    pub fn read(buf: &mut BytesMut) -> Result<Option<Header>, CodecError> {
        let data = match buf.get(0..HEADER_SIZE) {
            Some(data) => data,
            None => return Ok(None),
        };

        let mut data = Cursor::new(data);
        for byte in &PACKET_MAGIC[..] {
            if data.get_u8() != *byte {
                return Err(CodecError::BadData);
            }
        }

        let size = data.get_u16_le();
        let opcode = data.get_u8();

        Ok(Some(Header::new(opcode, size as usize)))
    }

    pub fn write(&self, buf: &mut BytesMut) {
        buf.reserve(HEADER_SIZE);

        buf.put_slice(&PACKET_MAGIC[..]);
        buf.put_u16_le(self.size as u16);
        buf.put_u8(self.opcode);
        buf.put_slice(&[0, 0, 0][..]);
    }
}