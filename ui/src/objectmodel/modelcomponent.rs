use std::cell::RefCell;
use std::rc;

use crate::{
    dimension::{
        Dimension,
        Position,
    }
};
use super::screensize::ScreenSize;

#[cfg(test)]
use crate::dimension::Offset;

pub struct ModelComponent {
    pub display: Option<ModelDisplay>,
    pub dimensions: ModelDimensions,
    pub parent: Option<rc::Weak<RefCell<ModelComponent>>>,
    pub children: Vec<rc::Rc<RefCell<ModelComponent>>>,
}

pub struct ModelDisplay {
    pub texture: rc::Rc<String>,
    pub texture_coords: ([f32; 2], [f32; 2]),
}

#[derive(Clone, Debug)]
pub struct ModelDimensions {
    pub top: f64,
    pub left: f64,
    pub right: f64,
    pub bottom: f64,
}

impl ModelDimensions {
    pub fn new(
        screen_size: &ScreenSize,
        position: &Position,
        size: &Dimension,
        parent: Option<&ModelDimensions>
    ) -> ModelDimensions
    {
        let parent = match parent {
            Some(parent) => parent.clone(),
            None => ModelDimensions {
                top: 0.0,
                left: 0.0,
                right: screen_size.width,
                bottom: screen_size.height,
            }
        };

        let parent_width = parent.right - parent.left;
        let parent_height = parent.bottom - parent.top;

        let mut top = parent.top;
        top += parent_height * position.y.relative as f64;
        top += position.y.pixels as f64;

        let mut left = parent.left;
        left += parent_width * position.x.relative as f64;
        left += position.x.pixels as f64;

        let width = parent_width * size.width.relative as f64 + size.width.pixels as f64;
        let height = parent_height * size.height.relative as f64 + size.height.pixels as f64;

        ModelDimensions {
            top,
            left,
            right: left + width,
            bottom: top + height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_modeldimensions_without_parent() {
        let screen_size = ScreenSize {
            width: 100.0,
            height: 100.0,
        };

        let dimensions = ModelDimensions::new(
            &screen_size,
            &Position::new(
                Offset::new(0.5, 0),
                Offset::new(0.0, 0),
            ),
            &Dimension::new(
                Offset::new(0.0, 10),
                Offset::new(0.0, 10),
            ),
            None
        );

        assert_eq!(dimensions.top, 0.0);
        assert_eq!(dimensions.left, 50.0);
        assert_eq!(dimensions.right, 60.0);
        assert_eq!(dimensions.bottom, 10.0);
    }

    #[test]
    fn test_new_modeldimensions_with_parent() {
        let screen_size = ScreenSize {
            width: 100.0,
            height: 100.0,
        };

        let parent = ModelDimensions::new(
            &screen_size,
            &Position::new(
                Offset::new(0.5, -10),
                Offset::new(0.5, -10),
            ),
            &Dimension::new(
                Offset::new(0.0, 20),
                Offset::new(0.0, 20),
            ),
            None
        );

        let dimensions = ModelDimensions::new(
            &screen_size,
            &Position::new(
                Offset::new(0.5, 0),
                Offset::new(0.0, 0),
            ),
            &Dimension::new(
                Offset::new(0.0, 10),
                Offset::new(0.0, 10),
            ),
            Some(&parent)
        );

        assert_eq!(dimensions.top, 40.0);
        assert_eq!(dimensions.left, 50.0);
        assert_eq!(dimensions.right, 60.0);
        assert_eq!(dimensions.bottom, 50.0);
    }
}