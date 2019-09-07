#[derive(Debug)]
pub struct Camera {
    pub view: nalgebra::Projective3<f32>,
    pub proj: nalgebra::Perspective3<f32>,
}

#[derive(Debug)]
pub struct Scene {
    pub camera: Camera,
}

impl Camera {
    pub fn new(aspect: f32) -> Camera {
        Camera {
            proj: nalgebra::Perspective3::new(
                aspect,
                3.1415 / 4.0, // FOV in radians?
                1.0,
                200.0,
            ),
            view: nalgebra::Projective3::identity(),
        }
    }

    pub fn set_view(&mut self, view: nalgebra::Projective3<f32>) {
        self.view = view;
    }
}