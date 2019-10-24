use std::cell::{
    Ref,
    RefCell,
};
use std::rc;

use crate::{
    Component,
    Element,
};
use super::{
    modelcomponent::{
        ModelComponent,
        ModelDimensions,
        ModelDisplay,
    },
    screensize::ScreenSize,
};

#[cfg(test)]
use crate::{
    dimension::{
        Dimension,
        Offset,
        Position,
    },
    element::ElementDisplay,
};

pub struct Tree {
    screen_size: ScreenSize,
    textures: Vec<rc::Weak<String>>,
    components: Vec<rc::Weak<RefCell<ModelComponent>>>,
    root: rc::Rc<RefCell<ModelComponent>>,
}

impl Tree {
    pub fn new(width: f64, height: f64, root: Box<dyn Component>) -> Tree {
        let screen_size = ScreenSize { width, height };

        let mut components = Vec::new();
        let mut textures = Vec::new();

        let element = root.render();
        let root_dimensions = ModelDimensions::new(
            &screen_size,
            &element.position,
            &element.size,
            None
        );
        let root_display = match element.display {
            Some(ref display) => {
                let texture = rc::Rc::new(display.texture.clone());
                textures.push(rc::Rc::downgrade(&texture));

                Some(ModelDisplay {
                    texture,
                    texture_coords: display.texture_coords,
                })
            },
            None => None,
        };

        let root = rc::Rc::new(RefCell::new(ModelComponent {
            dimensions: root_dimensions,
            display: root_display,
            parent: None,
            children: Vec::new(),
        }));
        components.push(rc::Rc::downgrade(&root));

        let mut tree = Tree {
            screen_size,
            components,
            textures,
            root: root.clone(),
        };

        for child in element.children {
            let mc = tree.render(child, root.clone());

            let mut parent = root.borrow_mut();
            parent.children.push(mc);
        }

        tree
    }
    
    #[cfg(test)]
    fn new_empty(width: f64, height: f64) -> Tree {
        let screen_size = ScreenSize { width, height };

        let root =  rc::Rc::new(RefCell::new(ModelComponent {
            display: None,
            parent: None,
            dimensions: ModelDimensions {
                top: 0.0,
                left: 0.0,
                right: width,
                bottom: height,
            },
            children: Vec::new(),
        }));
        let components = vec![rc::Rc::downgrade(&root)];

        Tree {
            screen_size,
            components,
            root,
            textures: Vec::new(),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<rc::Weak<RefCell<ModelComponent>>> {
        self.components.iter()
    }

    fn render(
        &mut self,
        component: Box<dyn Component>,
        parent: rc::Rc<RefCell<ModelComponent>>
    ) -> rc::Rc<RefCell<ModelComponent>> {
        let element = component.render();
        let mc = self.from_element(&element, rc::Rc::downgrade(&parent));

        for child in element.children {
            let child = self.render(child, mc.clone());

            let mut parent = mc.borrow_mut();
            parent.children.push(child);
        }

        mc
    }

    fn from_element(
        &mut self,
        element: &Element,
        parent: rc::Weak<RefCell<ModelComponent>>
    ) -> rc::Rc<RefCell<ModelComponent>>
    {
        let dimensions = ModelDimensions::new(
            &self.screen_size,
            &element.position,
            &element.size,
            Some(&parent.upgrade().unwrap().borrow().dimensions)
        );
        let display = match element.display {
            Some(ref display) => Some(ModelDisplay {
                texture: self.add_texture(display.texture.clone()),
                texture_coords: display.texture_coords,
            }),
            None => None,
        };

        let ret = rc::Rc::new(RefCell::new(ModelComponent {
            display,
            dimensions,
            parent: Some(parent),
            children: Vec::new(),
        }));
        self.components.push(rc::Rc::downgrade(&ret));

        ret
    }

    fn add_texture(&mut self, texture: String) -> rc::Rc<String> {
        for i in (0..self.textures.len()).rev() {
            match self.textures.get(i).unwrap().upgrade() {
                Some(tex) => {
                    if *tex == texture {
                        return tex;
                    }
                },
                None => {
                    self.textures.remove(i);
                },
            }
        }

        let tex = rc::Rc::new(texture);
        self.textures.push(rc::Rc::downgrade(&tex));

        tex
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_element() {
        let mut tree = Tree::new_empty(100.0, 100.0);
        assert_eq!(tree.components.len(), 1);

        let parent = tree.components.get(0).unwrap().clone();

        let elem = Element::new(
            Position::new(
                Offset::new(0.0, 0),
                Offset::new(0.0, 0),
            ),
            Dimension::new(
                Offset::new(0.0, 10),
                Offset::new(0.0, 10),
            ),
            None
        );

        tree.from_element(&elem, parent);
        assert_eq!(tree.components.len(), 2);
        assert_eq!(tree.textures.len(), 0);
    }

    #[test]
    fn test_from_element_with_texture() {
        let mut tree = Tree::new_empty(100.0, 100.0);
        assert_eq!(tree.components.len(), 1);

        let parent = tree.components.get(0).unwrap().clone();

        let elem = Element::new(
            Position::new(
                Offset::new(0.0, 0),
                Offset::new(0.0, 0),
            ),
            Dimension::new(
                Offset::new(0.0, 10),
                Offset::new(0.0, 10),
            ),
            Some(ElementDisplay::new(
                "tex1".to_string(),
                [0.0, 0.0],
                [1.0, 1.0]
            ))
        );

        let mc = tree.from_element(&elem, parent);

        let mc_ref = mc.borrow();
        assert!(mc_ref.display.is_some());

        let mc_tex = mc_ref.display.as_ref().unwrap();
        assert_eq!(&(*mc_tex.texture)[..], "tex1");
        assert_eq!(mc_tex.texture_coords.0, [0.0, 0.0]);
        assert_eq!(mc_tex.texture_coords.1, [1.0, 1.0]);

        assert_eq!(tree.components.len(), 2);
        assert_eq!(tree.textures.len(), 1);

        let tex = tree.textures.get(0).unwrap().upgrade();
        assert!(tex.is_some());
        assert_eq!(&(*tex.unwrap())[..], "tex1");
    }

    #[test]
    fn test_add_texture() {
        let mut tree = Tree::new_empty(100.0, 100.0);
        assert_eq!(tree.textures.len(), 0);

        let tex_ref = tree.add_texture("tex1".to_string());
        assert_eq!(tree.textures.len(), 1);

        let tex = tree.textures.get(0).unwrap().upgrade();
        assert!(tex.is_some());

        assert_eq!(*tex.unwrap(), *tex_ref);
    }
}