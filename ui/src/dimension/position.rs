use super::Offset;

pub struct Position {
    pub x: Offset,
    pub y: Offset,
}

impl Position {
    pub fn new(x: Offset, y: Offset) -> Position {
        Position { x, y }
    }
}