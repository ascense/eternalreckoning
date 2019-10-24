use super::Offset;

#[derive(Clone)]
pub struct Dimension {
    pub width: Offset,
    pub height: Offset,
}

impl Dimension {
    pub fn new(width: Offset, height: Offset) -> Dimension {
        Dimension { width, height }
    }
}