use crate::{
    dimension::{
        Dimension,
        Position,
    },
    Component,
};

pub struct Element {
    pub display: Option<ElementDisplay>,
    pub position: Position,
    pub size: Dimension,
    pub children: Vec<Box<dyn Component>>,
}

pub struct ElementDisplay {
    pub texture: String,
    pub texture_coords: ([f32; 2], [f32; 2]),
}

impl Element {
    pub fn new(
        position: Position,
        size: Dimension,
        display: Option<ElementDisplay>,
    ) -> Element
    {
        Element {
            display,
            position,
            size,
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: Box<dyn Component>) {
        self.children.push(child);
    }
}

impl ElementDisplay {
    pub fn new(
        texture: String,
        top_left: [f32; 2],
        bottom_right: [f32; 2]
    ) -> ElementDisplay
    {
        ElementDisplay {
            texture,
            texture_coords: (top_left, bottom_right),
        }
    }
}