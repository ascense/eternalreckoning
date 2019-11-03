#[derive(Copy, Clone)]
pub struct Offset {
    pub pixels: i32,
    pub relative: f32,
}

impl Offset {
    pub fn new(relative: f32, pixels: i32) -> Offset {
        Offset { relative, pixels }
    }
}