pub struct Packet {
    pub operation: Operation,
}

#[derive(PartialEq)]
pub enum Operation {
    ConnectReq = 0x01,
    ConnectRes = 0x02,
}

impl Operation {
    pub fn from_value(value: u8) -> Option<Operation> {
        match value {
            0x01 => Some(Operation::ConnectReq),
            0x02 => Some(Operation::ConnectRes),
            _ => None,
        }
    }
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", match self {
            Operation::ConnectReq => "connection request",
            Operation::ConnectRes => "connection response",
        })
    }
}