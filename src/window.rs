use rendy::wsi::winit;

pub struct Window {
    window: winit::Window,
    event_loop: winit::EventsLoop,
}

impl Window {
    pub fn new() -> Window {
        let mut event_loop = winit::EventsLoop::new();

        let window = winit::WindowBuilder::new()
            .with_title("World Client")
            .build(&event_loop)
            .unwrap();

        event_loop.poll_events(|_| ());
        
        Window { window, event_loop }
    }

    pub fn get_size(&self) -> winit::dpi::PhysicalSize {
        self.window
            .get_inner_size()
            .unwrap()
            .to_physical(self.window.get_hidpi_factor())
    }

    pub fn get_aspect_ratio(&self) -> f64 {
        let size = self.get_size();

        size.width / size.height
    }

    pub fn create_surface<B>(
        &self,
        factory: &mut rendy::factory::Factory<B>,
    ) -> rendy::wsi::Surface<B>
    where
        B: rendy::hal::Backend,
    {
        factory.create_surface(&self.window)
    }

    pub fn poll_events<F>(&mut self, callback: F)
    where
        F: FnMut(winit::Event),
    {
        self.event_loop.poll_events(callback);
    }
}